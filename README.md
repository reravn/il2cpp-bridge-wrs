# il2cpp-bridge-rs

<a href="https://crates.io/crates/il2cpp-bridge-rs"><img src="https://img.shields.io/crates/v/il2cpp-bridge-rs?style=for-the-badge" alt="Crates.io"></a>
<a href="#supported-targets"><img src="https://img.shields.io/badge/platform-iOS%20%C2%B7%20macOS%20%C2%B7%20Linux%20%C2%B7%20Android%20%C2%B7%20Windows-222?style=for-the-badge&logoColor=white" alt="Platform"></a>

`il2cpp-bridge-rs` is a Rust library for exploring and interacting with Unity's IL2CPP runtime from native code. It resolves IL2CPP exports at runtime, builds a metadata cache, and exposes ergonomic wrappers for common tasks such as class lookup, method invocation, metadata dumping, and Unity object access.

## Who This Is For

This crate is aimed at engineers who are already working inside a native environment where Unity and IL2CPP are loaded:

- native plugins and injected libraries
- runtime tooling and diagnostics
- IL2CPP introspection utilities
- Unity wrappers built on top of raw IL2CPP metadata

If you need a general Unity mod loader or a standalone game patching framework, this crate is a building block rather than a complete solution.

## Runtime Assumptions

Before using the API, keep these constraints in mind:

- A live IL2CPP runtime must already be loaded in the current process.
- You must call [`init`](src/init.rs) before relying on cache-backed lookups such as `api::cache::csharp()`.
- Many operations ultimately work with raw pointers and runtime-owned memory. The crate reduces footguns, but it does not make IL2CPP integration fully safe.
- Method calls, field access, and object wrappers only make sense while the target Unity runtime is alive and compatible with the expected metadata.
- Some operations require the current thread to be attached to the IL2CPP VM. Use `api::Thread` when you are doing work outside the initialization callback.

## Installation

```bash
cargo add il2cpp-bridge-rs
```

## Build and Validation

The repo includes a `Makefile` with common workflows:

```bash
make build
make check
make clippy
make doc
```

Platform-specific targets are also available, including `make build-ios`, `make build-macos`, `make build-linux`, `make build-android`, and `make build-windows`.

## First Successful Flow

The snippet below shows the intended happy path: initialize the runtime, fetch a cached assembly, resolve a class, inspect a method, and call an instance method if an object is available.

This example is illustrative. It compiles as Rust, but it requires a live Unity IL2CPP runtime to do anything useful.

```rust
use il2cpp_bridge_rs::{api, init};
use std::ffi::c_void;

init("GameAssembly", || {
    let asm = api::cache::csharp();
    let player_class = asm
        .class("PlayerController")
        .expect("PlayerController should exist after cache hydration");

    let damage_method = player_class
        .method(("TakeDamage", ["System.Single"]))
        .expect("TakeDamage(float) should exist");

    println!(
        "Resolved {}::{} at RVA 0x{:X}",
        player_class.name,
        damage_method.name,
        damage_method.rva
    );

    if let Some(player) = player_class.find_objects_of_type(false).into_iter().next() {
        let bound_method = player
            .method(("TakeDamage", ["System.Single"]))
            .expect("instance method should bind automatically");

        let damage: f32 = 25.0;
        unsafe {
            let _: Result<(), _> =
                bound_method.call(&[&damage as *const f32 as *mut c_void]);
        }
    }
});
```

## Documentation Map

The markdown guides explain workflows and caveats. Generated rustdoc should be treated as the source of truth for exact signatures.

- [Getting Started](docs/getting-started.md)
- [Core Workflows](docs/core-workflows.md)
- [Platform and Runtime Notes](docs/platform-support.md)
- [Architecture](docs/architecture.md)
- [API Map](docs/api-reference.md)

To build rustdoc locally:

```bash
cargo doc --no-deps
```

## Primary Entry Points

Most users will spend time in these APIs first:

- `init`
- `api::cache`
- `api::Thread`
- `api::invoke_method`
- dump helpers such as `api::dump` and `api::dump_all_to`
- wrappers such as `api::Application` and `api::Time`
- metadata and object wrappers under `structs`

## Supported Targets

The project currently ships build targets for:

- iOS: `aarch64-apple-ios`
- macOS: `aarch64-apple-darwin`
- Linux: `x86_64-unknown-linux-gnu`
- Android: `aarch64-linux-android`
- Windows: `x86_64-pc-windows-msvc`

See [Platform and Runtime Notes](docs/platform-support.md) for symbol resolution details, target image naming guidance, and thread/runtime caveats.

## Contributing

Contribution expectations and local development workflow are documented in [CONTRIBUTING.md](CONTRIBUTING.md).

# Legal Disclaimer
This library is provided strictly for educational and research purposes. It is intended to facilitate learning about IL2CPP internals, reverse engineering concepts, and Rust-based interop with native applications.

## License

[MIT](LICENSE)
