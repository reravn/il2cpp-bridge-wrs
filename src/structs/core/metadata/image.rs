//! IL2CPP Image definition
use crate::structs::core::Class;
use crate::api::{self, cache};
use std::ffi::c_void;

use super::assembly::Assembly;

/// Represents an IL2CPP Image (essentially a DLL or executable module)
#[derive(Debug, Clone)]
pub struct Image {
    /// Pointer to the internal IL2CPP image structure
    pub address: *mut c_void,
    /// Name of the image (e.g., "Assembly-CSharp")
    pub name: String,
    /// Filename of the image incl. extension (e.g., "Assembly-CSharp.dll")
    pub filename: String,
    /// Pointer to the assembly containing this image
    pub assembly: *mut c_void,
    /// Pointer to the entry point method (if any)
    pub entry_point: *mut c_void,
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}

impl Image {
    /// Gets the assembly corresponding to this image
    ///
    /// # Returns
    /// * `Option<Arc<Assembly>>` - The assembly containing this image, or None if not found
    pub fn get_assembly(&self) -> Option<std::sync::Arc<Assembly>> {
        unsafe {
            let assembly_ptr = api::image_get_assembly(self.address);
            if assembly_ptr.is_null() {
                return None;
            }

            for entry in cache::CACHE.assemblies.iter() {
                if entry.value().address == assembly_ptr {
                    return Some(std::sync::Arc::clone(&entry.value()));
                }
            }
            None
        }
    }

    /// Gets all classes defined in this image
    ///
    /// # Returns
    /// * `Vec<Class>` - A list of all classes defined in this image
    pub fn get_classes(&self) -> Vec<Class> {
        let mut classes = Vec::new();
        unsafe {
            let class_count = api::image_get_class_count(self.address);
            for i in 0..class_count {
                let class_ptr = api::image_get_class(self.address, i);
                if !class_ptr.is_null() {
                    if let Some(class) = cache::class_from_ptr(class_ptr) {
                        classes.push(class);
                    }
                }
            }
        }
        classes
    }

    /// Finds a class by name within this image
    ///
    /// # Arguments
    /// * `name` - The full name of the class (e.g., "UnityEngine.GameObject")
    ///
    /// # Returns
    /// * `Option<Class>` - The found class, or None if not found
    pub fn class(&self, name: &str) -> Option<Class> {
        if let Some(class) = cache::CACHE.classes.get(name) {
            return Some((**class).clone());
        }

        let (namespace, class_name) = if let Some(idx) = name.rfind('.') {
            (&name[..idx], &name[idx + 1..])
        } else {
            ("", name)
        };

        unsafe {
            let ns_cstr = std::ffi::CString::new(namespace).ok()?;
            let name_cstr = std::ffi::CString::new(class_name).ok()?;

            let class_ptr =
                api::class_from_name(self.address, ns_cstr.as_ptr(), name_cstr.as_ptr());

            if !class_ptr.is_null() {
                cache::class_from_ptr(class_ptr)
            } else {
                None
            }
        }
    }
}
