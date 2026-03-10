//! Initialization of the IL2CPP runtime and metadata cache.
//!
//! [`init`] is the front door to the crate. It resolves IL2CPP exports, loads
//! the function table, attaches the worker thread to the runtime, initializes
//! the cache, hydrates metadata, and then runs queued callbacks.
use crate::api::{self, cache, Thread};
use crate::memory::symbol::resolve_symbol;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

#[cfg(dev_release)]
use crate::logger;

/// The target image name, set once during [`init()`](super::init).
pub(crate) static TARGET_IMAGE_NAME: OnceLock<String> = OnceLock::new();

type Callback = Box<dyn FnOnce() + Send + 'static>;

enum State {
    /// No initialization has started yet.
    Idle,
    /// Background thread is running; callbacks queued here fire when it finishes.
    Running(Vec<Callback>),
    /// Initialization completed successfully; new callers fire immediately.
    Done,
}

static STATE: Mutex<State> = Mutex::new(State::Idle);
const CACHE_INIT_MAX_ATTEMPTS: u8 = 5;
const CACHE_INIT_RETRY_DELAY: Duration = Duration::from_secs(3);

/// Initializes IL2CPP symbol loading and cache hydration.
///
/// This function must be called before using cache-backed helpers such as
/// [`crate::api::cache::csharp`] or class/method lookups that depend on the
/// hydrated metadata cache.
///
/// Behavior:
///
/// - First call: spawns the initialization worker and queues `on_complete`.
/// - Calls while initialization is running: queue additional callbacks.
/// - Calls after successful initialization: execute `on_complete` immediately
///   on a newly spawned thread.
/// - On failure: the internal state resets to idle so initialization can be
///   attempted again.
///
/// The `target_image` should be the loaded module used to compute image base
/// addresses and method RVA/VA information. Common values include
/// `UnityFramework` on iOS and `GameAssembly` on many desktop Unity builds.
///
/// # Example
///
/// ```no_run
/// use il2cpp_bridge_rs::{api, init};
///
/// init("GameAssembly", || {
///     let asm = api::cache::csharp();
///     println!("Ready: {}", asm.name);
/// });
/// ```
///
/// # Parameters
///
/// - `target_image`: name of the loaded image backing RVA/VA calculations
/// - `on_complete`: callback executed after a successful initialization pass
pub fn init<F>(target_image: &str, on_complete: F)
where
    F: FnOnce() + Send + 'static,
{
    TARGET_IMAGE_NAME
        .set(target_image.to_owned())
        .unwrap_or_else(|_| {
            #[cfg(dev_release)]
            logger::info("TARGET_IMAGE_NAME already set, ignoring new value.");
        });
    let mut guard = STATE.lock().unwrap();

    match &mut *guard {
        State::Done => {
            drop(guard);
            std::thread::spawn(on_complete);
        }
        State::Running(callbacks) => {
            callbacks.push(Box::new(on_complete));
        }
        State::Idle => {
            *guard = State::Running(vec![Box::new(on_complete)]);
            drop(guard);

            std::thread::spawn(move || {
                if let Err(missing) = api::load(|symbol| match resolve_symbol(symbol) {
                    Ok(addr) => addr as *mut c_void,
                    Err(_e) => {
                        #[cfg(dev_release)]
                        logger::error(&format!("{}", _e));
                        std::ptr::null_mut()
                    }
                }) {
                    #[cfg(dev_release)]
                    logger::error(&format!(
                        "Failed to load IL2CPP API symbols: {}",
                        missing.join(", ")
                    ));

                    let mut guard = STATE.lock().unwrap();
                    *guard = State::Idle;
                    return;
                }

                #[cfg(dev_release)]
                logger::info("IL2CPP API loaded, waiting for runtime...");

                #[cfg(dev_release)]
                logger::info("IL2CPP runtime ready, installing early hook...");

                let _thread = Thread::attach(true);

                let mut cache_ready = false;
                for attempt in 1..=CACHE_INIT_MAX_ATTEMPTS {
                    if cache::init() {
                        cache_ready = true;
                        break;
                    }

                    if attempt < CACHE_INIT_MAX_ATTEMPTS {
                        #[cfg(dev_release)]
                        logger::info(&format!(
                            "Cache initialization attempt {}/{} failed. Retrying in {}s...",
                            attempt,
                            CACHE_INIT_MAX_ATTEMPTS,
                            CACHE_INIT_RETRY_DELAY.as_secs()
                        ));
                        thread::sleep(CACHE_INIT_RETRY_DELAY);
                    }
                }

                if cache_ready {
                    #[cfg(dev_release)]
                    logger::info("Cache ready, starting hooks...");

                    let callbacks = {
                        let mut guard = STATE.lock().unwrap();
                        let old = std::mem::replace(&mut *guard, State::Done);
                        match old {
                            State::Running(cbs) => cbs,
                            _ => vec![],
                        }
                    };

                    cache::ensure_hydrated();

                    std::thread::spawn(move || {
                        for cb in callbacks {
                            cb();
                        }
                    });
                } else {
                    #[cfg(dev_release)]
                    logger::error("Cache initialization failed after all retry attempts.");

                    let mut guard = STATE.lock().unwrap();
                    *guard = State::Idle;
                }
            });
        }
    }
}
