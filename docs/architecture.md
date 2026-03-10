# Architecture

## Module Overview

```
il2cpp-resolver-rs
├── init            Entry point — spawns background thread, manages callbacks
├── api             IL2CPP runtime interface
│   ├── core
│   │   ├── api         FFI bindings (200+ functions via macro)
│   │   ├── cache       Thread-safe assembly/class/method cache
│   │   ├── internals   Internal call resolution
│   │   └── runtime
│   │       ├── thread    RAII VM thread attachment
│   │       └── caller    Method invocation with exception handling
│   ├── debugging
│   │   └── cs          Metadata dump to string/file
│   └── wrappers
│       ├── application UnityEngine.Application properties
│       └── time        Time utilities
├── memory          Low-level memory operations
│   ├── rw            Generic pointer read/write
│   └── info
│       ├── symbol      Symbol resolution (per-platform)
│       └── image       Image base address lookup (per-platform)
├── structs         Type definitions mirroring IL2CPP/Unity
│   ├── core          Class, Object, Method, Field, Property, Assembly, Image, Type
│   ├── collections   Il2cppString, Il2cppArray, Il2cppList, Il2cppDictionary
│   ├── components    GameObject, Transform, Camera, Rigidbody, etc.
│   └── math          Vector2/3/4, Quaternion, Color, Matrix types
└── logger          Debug-only logging (gated by #[cfg(dev_release)])
```

## Data Flow

```
init(target_image, callback)
  │
  ├─ memory::symbol::resolve_symbol()    Resolve each IL2CPP export by name
  │     └─ Platform: dlsym / GetProcAddress
  │
  ├─ api::load()                         Populate function pointer table
  │     └─ define_il2cpp_functions! macro generates 200+ bindings
  │
  ├─ api::Thread::attach()               Attach to the IL2CPP VM
  │     └─ Auto-detaches on drop (RAII)
  │
  ├─ api::cache::init()                  Build assembly cache (with retries)
  │     ├─ domain_get() → domain_get_assemblies()
  │     └─ Hydrate Assembly structs into DashMap
  │
  ├─ api::cache::ensure_hydrated()       Hydrate all classes (deferred)
  │     └─ Populates fields, methods, properties for each class
  │
  └─ callback()                          Fire queued callbacks
```

## Key Patterns

### Macro-Generated FFI

`define_il2cpp_functions!` in `src/api/core/api.rs` generates:
- Type aliases for each function signature
- A struct holding all function pointers
- A `load()` function that resolves symbols and populates the struct
- Safe wrapper functions for each binding

### Thread-Safe Caching

`DashMap`-based concurrent cache in `src/api/core/cache.rs`:
- Assemblies stored as `Arc<Assembly>` for cheap cloning across threads
- Classes stored as `Box<Class>` with full metadata
- Methods indexed by `ClassName::MethodName` key
- Hydration is deferred and runs once via `AtomicBool` guard

### Platform Abstraction

`memory/info/symbol.rs` and `memory/info/image.rs` use `#[cfg]` blocks to select platform-specific implementations:
- **macOS/iOS**: `dlsym`, `dyld` APIs via `mach2`
- **Linux/Android**: `dlsym`, `dl_iterate_phdr` via `libc`
- **Windows**: `GetProcAddress`, `GetModuleHandleA` via `windows-sys`

### RAII Thread Management

`Thread` in `src/api/core/runtime/thread.rs`:
- `Thread::attach(auto_detach)` attaches to the IL2CPP GC domain
- When `auto_detach` is `true`, the thread detaches on `Drop`
