//! IL2CPP struct definitions and submodules
//!
//! This module contains the core structure definitions for the IL2CPP interaction layer.
//! It mirrors Unity's internal structures and provides safe wrappers where possible.

/// Collection types (List, Array, Dictionary, String)
pub mod collections;
/// Unity Component wrappers (GameObject, Transform, etc.)
pub mod components;
/// Core IL2CPP structures (Class, Method, Image, etc.)
pub mod core;
/// Mathematical structures (Vector3, Quaternion, etc.)
pub mod math;
/// Pointer utilities
pub mod ptr;

// Re-exports for easier access
pub use collections::Il2cppString;
pub use components::GameObject;
pub use components::MonoBehaviour;
pub use components::Transform;
pub use core::{
    Arg, Assembly, Class, Coroutine, Field, Image, Method, Object, Property, Type, ValueType,
};
pub use math::{Color, Matrix2x2, Matrix3x3, Matrix4x4, Quaternion, Vector2, Vector3, Vector4};
pub use ptr::MutPtr;
