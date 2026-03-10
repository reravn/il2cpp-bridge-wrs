#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod api;
mod init;
pub mod logger;
pub mod memory;
pub mod structs;

pub use init::init;