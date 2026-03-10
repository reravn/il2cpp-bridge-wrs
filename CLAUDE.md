# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                                  # Build for host platform (debug)
cargo build --target aarch64-apple-ios       # Build for iOS
cargo build --release                        # Build for host platform (release)
cargo check                                  # Type-check without building
cargo check --target aarch64-apple-ios       # Type-check for iOS
```

No test suite exists. No linter configuration beyond `cargo clippy`.

## Architecture

Rust library crate for Unity IL2CPP runtime introspection. Cross-platform: macOS/iOS, Linux/Android, Windows.

### Module Overview

- **`init`** — Entry point. `init(on_complete)` spawns a background thread that loads IL2CPP symbols, attaches to the VM, initializes the cache with retries, then fires queued callbacks.
- **`api`** — IL2CPP runtime interface. Contains FFI bindings, caching, thread management, method invocation, and debugging/SDK generation tools.
- **`memory`** — Low-level memory operations: `rw` (generic ptr read/write), `info::symbol` (symbol resolution), `info::image` (loaded image base address lookup).
- **`structs`** — Type wrappers mirroring Unity/IL2CPP: core hierarchy (Class, Object, Type), members (Field, Method, Property), metadata (Assembly, Image), collections (Array, List, Dictionary, String), components (GameObject, Transform, Camera, etc.), and math types.
- **`config`** — `TARGET_IMAGE_NAME` constant used for RVA/VA calculations.
- **`logger`** — Logging stubs, only active in debug builds via `#[cfg(dev_release)]`.

### Key Patterns

**Macro-generated FFI**: `define_il2cpp_functions!` in `api/core/api.rs` generates 200+ IL2CPP function bindings — type aliases, a struct of function pointers, a `load()` initializer, and safe wrapper functions.

**Thread-safe caching**: `DashMap`-based concurrent cache in `api/core/cache.rs` stores assemblies (`Arc<Assembly>`), classes (`Box<Class>`), and methods. Symbol and image lookups in `memory/info/` also cache results.

**Platform abstraction via `#[cfg]` modules**: `memory/info/image.rs` and `memory/info/symbol.rs` use per-platform `mod platform` blocks (macOS/iOS, Linux/Android, Windows, fallback) for native API calls (dyld, dl_iterate_phdr, GetModuleHandleA, dlsym, GetProcAddress).

**RAII thread management**: `Thread` struct in `api/core/runtime/thread.rs` auto-detaches from the IL2CPP VM on drop.

### Data Flow

```
init::init(callback)
  → memory::info::symbol::resolve_symbol() for each IL2CPP export
  → api::api::load() populates function pointer table
  → api::Thread::attach() to IL2CPP VM
  → api::cache::init() builds assembly/class/method cache
  → callback()
```

At runtime, use `cache::assembly()` / `cache::class()` to look up types, then `caller::invoke_method()` for reflection-based calls with managed exception handling.

### Conditional Compilation

- `#[cfg(dev_release)]` — Enabled in debug builds by `build.rs`. Gates all logging calls.
- Platform-specific dependencies: `libc` (unix), `mach2` (macOS/iOS), `windows-sys` (Windows).