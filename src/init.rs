//! Initialization of the IL2CPP runtime and cache
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

/// Initializes the IL2CPP runtime and its internal cache.
///
/// - First call: spawns the background init thread and queues `on_complete`.
/// - Calls while the thread is running: queue `on_complete`; it fires when the
///   thread finishes.
/// - Calls after init is done: dispatch `on_complete` to the main thread
///   immediately.
///
/// # Type Parameters
/// * `F` - A closure that runs when initialization is complete. Must be `Send` and `'static`.
///
/// # Arguments
/// * `on_complete` - The callback to execute after successful initialization.
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
                api::load(|symbol| match resolve_symbol(symbol) {
                    Ok(addr) => addr as *mut c_void,
                    Err(_e) => {
                        #[cfg(dev_release)]
                        logger::error(&format!("{}", _e));
                        std::ptr::null_mut()
                    }
                });

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
                    *guard = State::Done;
                }
            });
        }
    }
}
