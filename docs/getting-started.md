# Getting Started

## Installation

```bash
cargo add il2cpp-resolver-rs
```

## Building

A `Makefile` is provided with per-platform targets (`build-*`, `build-*-release`, `check-*`, `clippy-*`):

```bash
make build-ios          # iOS (aarch64-apple-ios) — primary target
make build-macos        # macOS (aarch64-apple-darwin)
make build-linux        # Linux (x86_64-unknown-linux-gnu)
make build-android      # Android (aarch64-linux-android)
make build-windows      # Windows (x86_64-pc-windows-msvc)
make build-ios-release  # Release build for iOS
```

Run `make` with no arguments for a host-platform debug build. See the [README](../README.md#building) for the full list of targets.

## Initialization

Call `init()` with the target image name and a callback. The library spawns a background thread that:

1. Resolves IL2CPP exported symbols
2. Loads the FFI function pointer table
3. Attaches to the IL2CPP VM thread
4. Builds the assembly/class/method cache
5. Fires your callback

```rust
use il2cpp_resolver_rs::{init, api};

init("UnityFramework", || {
    println!("IL2CPP ready!");
});
```

The `target_image` parameter is the name of the loaded binary (e.g. `"UnityFramework"` on iOS, `"GameAssembly"` on desktop). It's used to compute RVA/VA addresses.

Multiple calls to `init()` before completion queue callbacks. Calls after completion dispatch immediately.

## Looking Up Classes

```rust
use il2cpp_resolver_rs::api::cache;

// Get Assembly-CSharp (most game code lives here)
let asm = cache::csharp();

// Find a class by name
if let Some(class) = asm.class("PlayerController") {
    println!("Class: {} (namespace: {})", class.name, class.namespace);
    println!("Fields: {}, Methods: {}", class.fields.len(), class.methods.len());
}

// Other common assemblies
let mscorlib = cache::mscorlib();
let core = cache::coremodule();

// Any assembly by name
let asm = cache::assembly("UnityEngine.UI");
```

## Finding Methods and Fields

```rust
if let Some(class) = asm.class("PlayerHealth") {
    // Find a method
    if let Some(method) = class.method("TakeDamage") {
        println!("RVA: {:#x}, VA: {:#x}", method.rva, method.va);
        println!("Static: {}, Params: {}", method.is_static, method.param_count);

        for arg in &method.args {
            println!("  param: {} ({})", arg.name, arg.type_info.name);
        }
    }

    // Find a field
    if let Some(field) = class.field("health") {
        println!("Offset: {:#x}, Type: {}", field.offset, field.type_info.name);
        println!("Static: {}, ReadOnly: {}", field.is_static, field.is_readonly);
    }

    // Properties (derived from get_/set_ method pairs)
    for prop in &class.properties {
        println!("Property: {} ({})", prop.name, prop.type_name);
    }
}
```

## Invoking Methods

Use `Method::call<T>()` to invoke methods. It handles instance binding, argument validation, value type unboxing, and exception handling for you.

```rust
use std::ffi::c_void;

// Static method — call directly from a class lookup
if let Some(method) = class.method("GetInstance") {
    unsafe {
        let player: Result<*mut c_void, _> = method.call(&[]);
    }
}

// Instance method — use Object::method() to auto-bind the instance
let obj = unsafe { Object::from_ptr(some_ptr) };
if let Some(method) = obj.method("TakeDamage") {
    let damage: f32 = 25.0;
    unsafe {
        let result: Result<(), _> = method.call(&[
            &damage as *const f32 as *mut c_void,
        ]);
    }
}
```

Managed exceptions are caught automatically and returned as `Err(String)` with the exception message.

### Method Selectors

When a class has overloaded methods, you can disambiguate with selectors:

```rust
// By name only
class.method("Update")

// By name + parameter type names
class.method(("TakeDamage", &["System.Single"][..]))

// By name + parameter count
class.method(("TakeDamage", 1))
```

### Low-Level: `invoke_method`

For cases where you already have raw pointers:

```rust
use il2cpp_resolver_rs::api::invoke_method;

let result = invoke_method(method_ptr, obj_ptr, params_ptr);
```

## Finding Objects in the Scene

Use `Class::find_objects_of_type` to query all live instances of a type, just like Unity's `FindObjectsOfType`.

```rust
let asm = api::cache::csharp();
if let Some(class) = asm.class("PlayerController") {
    // Find all active PlayerController instances
    let players = class.find_objects_of_type(false);
    println!("Found {} players", players.len());

    // Call a method on each one
    for player in &players {
        if let Some(method) = player.method("Heal") {
            let amount: f32 = 100.0;
            unsafe {
                let _: Result<(), _> = method.call(&[
                    &amount as *const f32 as *mut c_void,
                ]);
            }
        }
    }

    // Pass true to include inactive objects
    let all_players = class.find_objects_of_type(true);
}
```

You can also find a specific `GameObject` by name:

```rust
use il2cpp_resolver_rs::structs::GameObject;

if let Ok(go) = GameObject::find("Player") {
    println!("Found: {:?}", go);
}
```

## Dumping Metadata

```rust
use il2cpp_resolver_rs::api;

// Dump a single assembly
if let Some(text) = api::dump("Assembly-CSharp") {
    println!("{}", text);
}

// Dump all assemblies to a directory
api::dump_all_to("/tmp/il2cpp_dump");
```

## Going Deeper

This guide covers the most common workflows, but there's a lot more available. The best way to learn the full API is to read the source:

- **`src/structs/`** — All type wrappers (`Class`, `Object`, `Method`, `Field`, `GameObject`, `Transform`, etc.) with their methods
- **`src/api/wrappers/`** — Real-world usage patterns (e.g. `Application`, `Time`) that show how to combine lookups, method calls, and type conversions
- **`src/api/core/`** — The caching layer, FFI bindings, and runtime internals
- **`src/memory/`** — Low-level memory read/write and platform-specific symbol resolution
