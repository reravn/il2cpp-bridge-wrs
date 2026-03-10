//! IL2CPP Core types exports

pub mod hierarchy;
pub mod members;
pub mod metadata;
pub mod runtime;

// Re-export types from submodules for flat access
pub use hierarchy::class::{Class, MethodSelector};
pub use hierarchy::object::{Il2cppObject, Object};
pub use hierarchy::r#type::Type;
pub use hierarchy::value_type::ValueType;

pub use members::field::Field;
pub use members::method::{Arg, Method};
pub use members::property::Property;

pub use metadata::assembly::Assembly;
pub use metadata::image::Image;

pub use runtime::coroutine::Coroutine;
