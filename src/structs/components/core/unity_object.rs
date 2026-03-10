//! Unity Object base wrapper
use crate::structs::components::Transform;
use crate::structs::core::{Class, Il2cppObject, Object};
use crate::structs::math::{Quaternion, Vector3};
use crate::structs::Il2cppString;
use crate::api::cache;
use std::ffi::c_void;
use std::ops::Deref;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UnityObject {
    /// Base Object structure
    pub object: Object,
    /// Cached pointer to the object
    pub m_cached_ptr: *mut c_void,
}

impl UnityObject {
    /// Creates a UnityObject from a raw pointer
    ///
    /// # Arguments
    /// * `ptr` - The raw pointer to the object
    ///
    /// # Returns
    /// * `Self` - The created UnityObject wrapper
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        let object = unsafe { Object::from_ptr(ptr) };
        let m_cached_ptr = unsafe {
            let offset = std::mem::size_of::<Il2cppObject>() as isize;
            *(ptr.offset(offset) as *mut *mut c_void)
        };
        Self {
            object,
            m_cached_ptr,
        }
    }

    /// Returns the raw pointer to the object
    ///
    /// # Returns
    /// * `*mut c_void` - The raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.object.as_ptr()
    }

    /// Gets the name of the object
    ///
    /// # Returns
    /// * `Result<String, String>` - The name of the object
    pub fn get_name(&self) -> Result<String, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let ptr = obj
                .method("get_name")
                .ok_or("Method 'get_name' not found")?
                .call::<*mut Il2cppString>(&mut [])?;

            if ptr.is_null() {
                return Err("Name is null".to_string());
            }

            (*ptr)
                .to_string()
                .ok_or_else(|| "Failed to convert name to String".to_string())
        }
    }

    /// Returns the string representation of the object
    ///
    /// # Returns
    /// * `Result<String, String>` - String representation
    pub fn to_string(&self) -> Result<String, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let ptr = obj
                .method("ToString")
                .ok_or("Method 'ToString' not found")?
                .call::<*mut Il2cppString>(&mut [])?;

            if ptr.is_null() {
                return Err("String is null".to_string());
            }

            (*ptr)
                .to_string()
                .ok_or_else(|| "Failed to convert String to String".to_string())
        }
    }
    /// Gets the UnityObject class definition
    ///
    /// # Returns
    /// * `Option<Class>` - The UnityEngine.Object class
    pub fn get_class() -> Option<Class> {
        cache::coremodule().class("Object")
    }

    /// Instantiates a copy of the object
    ///
    /// # Arguments
    /// * `original` - The object to clone
    ///
    /// # Returns
    /// * `Result<UnityObject, String>` - The cloned object
    pub fn instantiate(original: &UnityObject) -> Result<UnityObject, String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method(("Instantiate", 1))
                .ok_or("Method 'Instantiate' not found")?;

            let ptr = method.call::<*mut c_void>(&mut [original.as_ptr()])?;

            if ptr.is_null() {
                return Err("Failed to instantiate object".to_string());
            }

            Ok(UnityObject::from_ptr(ptr))
        }
    }

    /// Instantiates a copy of the object at position and rotation
    ///
    /// # Arguments
    /// * `original` - The object to clone
    /// * `position` - The position for the new object
    /// * `rotation` - The rotation for the new object
    ///
    /// # Returns
    /// * `Result<UnityObject, String>` - The cloned object
    pub fn instantiate_at(
        original: &UnityObject,
        position: Vector3,
        rotation: Quaternion,
    ) -> Result<UnityObject, String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method(("Instantiate", 3))
                .ok_or("Method 'Instantiate' not found")?;

            let ptr = method.call::<*mut c_void>(&mut [
                original.as_ptr(),
                &position as *const _ as *mut c_void,
                &rotation as *const _ as *mut c_void,
            ])?;

            if ptr.is_null() {
                return Err("Failed to instantiate object".to_string());
            }

            Ok(UnityObject::from_ptr(ptr))
        }
    }

    /// Instantiates a copy of the object with parent
    ///
    /// # Arguments
    /// * `original` - The object to clone
    /// * `parent` - The parent Transform for the new object
    ///
    /// # Returns
    /// * `Result<UnityObject, String>` - The cloned object
    pub fn instantiate_with_parent(
        original: &UnityObject,
        parent: &Transform,
    ) -> Result<UnityObject, String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method(("Instantiate", 2))
                .ok_or("Method 'Instantiate' not found")?;

            let ptr = method
                .call::<*mut c_void>(&mut [original.as_ptr(), parent.as_ptr() as *mut c_void])?;

            if ptr.is_null() {
                return Err("Failed to instantiate object".to_string());
            }

            Ok(UnityObject::from_ptr(ptr))
        }
    }

    /// Destroys the object after a delay
    ///
    /// # Arguments
    /// * `time_delay` - The time delay in seconds
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn destroy(&self, time_delay: f32) -> Result<(), String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method("Destroy")
                .ok_or("Method 'Destroy' not found")?;

            method
                .call::<c_void>(&mut [self.as_ptr(), &time_delay as *const f32 as *mut c_void])?;
            Ok(())
        }
    }

    /// Destroys the object immediately
    ///
    /// # Arguments
    /// * `obj` - The object to destroy
    /// * `allow_destroying_assets` - Whether to allow destroying assets
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn destroy_immediate(
        obj: &UnityObject,
        allow_destroying_assets: bool,
    ) -> Result<(), String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method("DestroyImmediate")
                .ok_or("Method 'DestroyImmediate' not found")?;

            method.call::<c_void>(&mut [
                obj.as_ptr(),
                &allow_destroying_assets as *const bool as *mut c_void,
            ])?;
            Ok(())
        }
    }

    /// Preserves the object during scene loading
    ///
    /// # Arguments
    /// * `obj` - The object to preserve
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn dont_destroy_on_load(obj: &UnityObject) -> Result<(), String> {
        unsafe {
            let object_class = Self::get_class()
                .ok_or_else(|| "Could not find UnityEngine.Object class".to_string())?;
            let method = object_class
                .method("DontDestroyOnLoad")
                .ok_or("Method 'DontDestroyOnLoad' not found")?;

            method.call::<c_void>(&mut [obj.as_ptr()])?;
            Ok(())
        }
    }
}

impl Deref for UnityObject {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
