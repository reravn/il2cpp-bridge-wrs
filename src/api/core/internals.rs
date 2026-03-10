//! Internal call resolution helper
use super::api;
use std::ffi::{c_void, CString};

pub struct Internals;

impl Internals {
    /// Resolves an internal call by name
    ///
    /// # Arguments
    /// * `name` - The name of the internal call to resolve
    ///
    /// # Returns
    /// * `*mut c_void` - Pointer to the resolved function, or null if not found
    pub fn resolve(name: &str) -> *mut c_void {
        let name_c = CString::new(name).unwrap();
        unsafe { api::resolve_icall(name_c.as_ptr()) }
    }

    /// Adds a new internal call
    ///
    /// # Arguments
    /// * `name` - The name of the internal call to register
    /// * `method` - Pointer to the function to register
    pub fn add(name: &str, method: *mut c_void) {
        let name_c = CString::new(name).unwrap();
        unsafe { api::add_internal_call(name_c.as_ptr(), method) }
    }
}
