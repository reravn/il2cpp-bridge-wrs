//! IL2CPP String definition and operations
use crate::api;
use std::os::raw::c_void;
use std::slice;

/// Represents an IL2CPP String
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Il2cppString {
    /// Pointer to the string class
    pub klass: *mut c_void,
    /// Monitor for synchronization
    pub monitor: *mut c_void,
    /// Length of the string (in characters, not bytes)
    pub length: i32,
    /// Start of the character array (UTF-16)
    pub chars: [u16; 1],
}

impl Il2cppString {
    /// Converts the Il2cppString to a Rust String
    ///
    /// # Returns
    /// * `Option<String>` - The converted Rust string, or None if invalid UTF-16
    pub fn to_string(&self) -> Option<String> {
        let len = self.length as usize;
        unsafe {
            let slice = slice::from_raw_parts(self.chars.as_ptr(), len);
            String::from_utf16(slice).ok()
        }
    }

    /// Creates a new Il2cppString from a Rust string slice
    ///
    /// # Arguments
    /// * `s` - The Rust string slice to convert
    ///
    /// # Returns
    /// * `*mut Il2cppString` - Pointer to the newly created IL2CPP string
    pub fn new(s: &str) -> *mut Il2cppString {
        let c_str = std::ffi::CString::new(s).unwrap();
        unsafe { api::string_new(c_str.as_ptr()) as *mut Il2cppString }
    }
}
