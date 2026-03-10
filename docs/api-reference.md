# API Reference

## `init(target_image, on_complete)`

Entry point. Initializes the IL2CPP runtime and cache on a background thread.

```rust
pub fn init<F>(target_image: &str, on_complete: F)
where F: FnOnce() + Send + 'static
```

- **First call**: spawns the init thread, queues `on_complete`
- **While running**: queues additional callbacks
- **After done**: dispatches callback immediately

---

## `api::cache`

Thread-safe assembly/class/method cache.

| Function | Returns | Description |
|----------|---------|-------------|
| `assembly(name)` | `Option<Arc<Assembly>>` | Look up assembly by name (with or without `.dll`) |
| `csharp()` | `Arc<Assembly>` | `Assembly-CSharp.dll` (panics if missing) |
| `mscorlib()` | `Arc<Assembly>` | `mscorlib.dll` (panics if missing) |
| `coremodule()` | `Arc<Assembly>` | `UnityEngine.CoreModule.dll` (panics if missing) |
| `class_from_ptr(ptr)` | `Option<Class>` | Hydrate a class from a raw pointer |
| `method_from_ptr(ptr)` | `Option<Method>` | Hydrate a method from a raw pointer |
| `ensure_hydrated()` | `()` | Hydrate all classes (runs once, safe to call multiple times) |

---

## Method Invocation

### `Method::call<T>` (recommended)

The primary way to invoke IL2CPP methods. Handles instance binding, argument count validation, value type unboxing, and exception handling.

```rust
pub unsafe fn call<T>(&self, params: &[*mut c_void]) -> Result<T, String>
```

- For **static methods** obtained via `Class::method()`, no instance is needed.
- For **instance methods**, use `Object::method()` which auto-binds the instance, or set `method.instance` manually.
- Managed exceptions are caught and returned as `Err(String)`.

### `api::invoke_method` (low-level)

Raw pointer interface when you already have method/object pointers:

```rust
pub fn invoke_method(
    method: *mut c_void,
    obj: *mut c_void,       // null for static methods
    params: *const *mut c_void,
) -> Result<*mut c_void, String>
```

---

## `api::Thread`

RAII wrapper for IL2CPP VM thread attachment.

```rust
let _thread = api::Thread::attach(true); // auto-detach on drop
```

---

## `api::Internals`

Resolve and register IL2CPP internal calls.

| Function | Description |
|----------|-------------|
| `Internals::resolve(name)` | Resolve an internal call by name |
| `Internals::add(name, method)` | Register a new internal call |

---

## Debugging / Dump

| Function | Description |
|----------|-------------|
| `api::dump(assembly_name)` | Dump single assembly metadata to `Option<String>` |
| `api::dump_assembly(assembly)` | Dump an `&Assembly` to `Option<String>` |
| `api::dump_all()` | Dump all assemblies, returns combined string |
| `api::dump_all_to(path)` | Dump all assemblies to files in a directory |
| `api::dump_to(path, name)` | Dump a single assembly to a file |

---

## Wrappers

High-level wrappers for Unity engine types.

### `api::Application`

| Method | Returns | Description |
|--------|---------|-------------|
| `data_path()` | `Result<String, String>` | Application data path |
| `identifier()` | `Result<String, String>` | Bundle identifier |
| `version()` | `Result<String, String>` | Application version |

### `api::Time`

Time-related utilities from `UnityEngine.Time`.

---

## Structs

### Core Types

| Type | Key Fields |
|------|-----------|
| `Assembly` | `name`, `image`, `classes`, `address` |
| `Class` | `name`, `namespace`, `fields`, `methods`, `properties`, `is_enum`, `is_abstract`, ... |
| `Method` | `name`, `rva`, `va`, `args`, `return_type`, `is_static`, `param_count` |
| `Field` | `name`, `offset`, `type_info`, `is_static`, `is_readonly`, `is_literal` |
| `Property` | `name`, `type_name`, `getter`, `setter` |
| `Type` | `name`, `size`, `address` |
| `Image` | `name`, `filename`, `address` |

### Lookup Methods on Types

- `Assembly::class(name) -> Option<Class>`
- `Class::method(selector) -> Option<Method>` — see selectors below
- `Class::field(name) -> Option<Field>`
- `Class::property(name) -> Option<Property>`
- `Object::method(selector) -> Option<Method>` — same as `Class::method` but auto-binds the instance
- `Object::field(name) -> Option<Field>` — auto-binds the instance
- `Object::property(name) -> Option<Property>`

### Method Selectors (`MethodSelector` trait)

| Selector | Example | Matches |
|----------|---------|---------|
| `&str` | `"Update"` | By name |
| `(&str, &[&str])` | `("TakeDamage", &["System.Single"][..])` | By name + parameter type names |
| `(&str, usize)` | `("TakeDamage", 1)` | By name + parameter count |

### Scene Queries

| Method | Returns | Description |
|--------|---------|-------------|
| `Class::find_objects_of_type(include_inactive)` | `Vec<Object>` | Find all instances of a type in the scene |
| `GameObject::find(name)` | `Result<GameObject, String>` | Find a `GameObject` by name |

### Collections

`Il2cppString`, `Il2cppArray`, `Il2cppList`, `Il2cppDictionary`

### Unity Components

`GameObject`, `Transform`, `MonoBehaviour`, `Component`, `Camera`, `Renderer`, `Material`, `Shader`, `Rigidbody`, `Collider`, `Animator`, `SceneManager`

### Math

`Vector2`, `Vector3`, `Vector4`, `Quaternion`, `Color`, `Matrix2x2`, `Matrix3x3`, `Matrix4x4`

---

## Memory

Low-level operations (typically not used directly).

| Module | Description |
|--------|-------------|
| `memory::rw` | Generic pointer read/write |
| `memory::symbol::resolve_symbol(name)` | Platform-specific symbol resolution |
| `memory::image::get_image_base(name)` | Platform-specific image base address |
