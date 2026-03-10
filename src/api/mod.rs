//! Utilities for interacting with IL2CPP
//!
//! This module provides a collection of utilities for working with IL2CPP,
//! including core functionality, debugging tools, and convenient wrappers.
//!
//! # Modules
//!
//! * `core` - essential low-level utilities and type definitions
//! * `debugging` - tools for analyzing and debugging IL2CPP applications
//! * `wrappers` - high-level wrappers for common IL2CPP types and operations

pub mod core;
pub use core::api::*;
pub use core::cache;
pub use core::caller::invoke_method;
pub use core::internals::{self, Internals};
pub use core::thread::Thread;

pub mod debugging;

pub use debugging::cs::{dump, dump_all, dump_all_to, dump_assembly, dump_to};

pub mod wrappers;
pub use wrappers::{application::Application, time::Time};
