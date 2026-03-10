//! Unity Transform component wrapper
// use crate::structs::core::{Il2cppObject, Object}; // Object is needed for valid calls
use crate::structs::core::{Il2cppObject, Object};
use crate::structs::math::{Quaternion, Vector3};
use std::ffi::c_void;

use super::component::ComponentTrait;
use super::game_object::GameObject;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Pointer to the internal IL2CPP object
    pub ptr: *mut Il2cppObject,
}

impl ComponentTrait for Transform {
    fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr: ptr as *mut Il2cppObject,
        }
    }
}

impl Transform {
    /// Creates a Transform from a raw pointer
    ///
    /// # Arguments
    /// * `ptr` - The raw pointer to the Transform
    ///
    /// # Returns
    /// * `Self` - The created Transform wrapper
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        <Self as ComponentTrait>::from_ptr(ptr)
    }

    /// Returns the raw pointer to the transform
    ///
    /// # Returns
    /// * `*mut c_void` - The raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr as *mut c_void
    }

    /// Gets the GameObject attached to this Transform
    ///
    /// # Returns
    /// * `Result<GameObject, String>` - The GameObject attached to this Transform
    pub fn get_game_object(&self) -> Result<GameObject, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let ptr = obj
                .method("get_gameObject")
                .ok_or("Method 'get_gameObject' not found")?
                .call::<*mut c_void>(&mut [])?;

            if ptr.is_null() {
                return Err("Transform.gameObject is null".to_string());
            }

            Ok(GameObject::from_ptr(ptr))
        }
    }

    /// Gets the world position of the transform
    ///
    /// # Returns
    /// * `Result<Vector3, String>` - The world position
    pub fn get_position(&self) -> Result<Vector3, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_position")
                .ok_or("Method 'get_position' not found")?
                .call::<Vector3>(&mut [])
        }
    }

    /// Sets the world position of the transform
    ///
    /// # Arguments
    /// * `value` - The new world position
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn set_position(&self, value: Vector3) -> Result<(), String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&value as *const Vector3 as *mut c_void];
            obj.method("set_position")
                .ok_or("Method 'set_position' not found")?
                .call::<()>(&mut params)
        }
    }

    /// Gets the local position of the transform
    ///
    /// # Returns
    /// * `Result<Vector3, String>` - The local position concerning the parent
    pub fn local_position(&self) -> Result<Vector3, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_localPosition")
                .ok_or("Method 'get_localPosition' not found")?
                .call::<Vector3>(&mut [])
        }
    }

    /// Sets the local position of the transform
    ///
    /// # Arguments
    /// * `value` - The new local position
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn set_local_position(&self, value: Vector3) -> Result<(), String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&value as *const Vector3 as *mut c_void];
            obj.method("set_localPosition")
                .ok_or("Method 'set_localPosition' not found")?
                .call::<()>(&mut params)
        }
    }

    /// Gets the world rotation of the transform
    ///
    /// # Returns
    /// * `Result<Quaternion, String>` - The world rotation
    pub fn get_rotation(&self) -> Result<Quaternion, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_rotation")
                .ok_or("Method 'get_rotation' not found")?
                .call::<Quaternion>(&mut [])
        }
    }

    /// Sets the world rotation of the transform
    ///
    /// # Arguments
    /// * `value` - The new world rotation
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn set_rotation(&self, value: Quaternion) -> Result<(), String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&value as *const Quaternion as *mut c_void];
            obj.method("set_rotation")
                .ok_or("Method 'set_rotation' not found")?
                .call::<()>(&mut params)
        }
    }

    /// Gets the local rotation of the transform
    ///
    /// # Returns
    /// * `Result<Quaternion, String>` - The local rotation relative to the parent
    pub fn local_rotation(&self) -> Result<Quaternion, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_localRotation")
                .ok_or("Method 'get_localRotation' not found")?
                .call::<Quaternion>(&mut [])
        }
    }

    /// Sets the local rotation of the transform
    ///
    /// # Arguments
    /// * `value` - The new local rotation
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn set_local_rotation(&self, value: Quaternion) -> Result<(), String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&value as *const Quaternion as *mut c_void];
            obj.method("set_localRotation")
                .ok_or("Method 'set_localRotation' not found")?
                .call::<()>(&mut params)
        }
    }

    /// Gets the local scale of the transform
    ///
    /// # Returns
    /// * `Result<Vector3, String>` - The local scale
    pub fn get_local_scale(&self) -> Result<Vector3, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_localScale")
                .ok_or("Method 'get_localScale' not found")?
                .call::<Vector3>(&mut [])
        }
    }

    /// Sets the local scale of the transform
    ///
    /// # Arguments
    /// * `value` - The new local scale
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success
    pub fn set_local_scale(&self, value: Vector3) -> Result<(), String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&value as *const Vector3 as *mut c_void];
            obj.method("set_localScale")
                .ok_or("Method 'set_localScale' not found")?
                .call::<()>(&mut params)
        }
    }

    /// Gets the lossy scale (global scale) of the transform
    ///
    /// # Returns
    /// * `Result<Vector3, String>` - The global scale (approximate)
    pub fn get_lossy_scale(&self) -> Result<Vector3, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            obj.method("get_lossyScale")
                .ok_or("Method 'get_lossyScale' not found")?
                .call::<Vector3>(&mut [])
        }
    }

    /// Gets the number of children
    ///
    /// # Returns
    /// * `i32` - The number of child transforms
    pub fn get_child_count(&self) -> i32 {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            if let Some(method) = obj.method("get_childCount") {
                return method.call::<i32>(&mut []).unwrap_or(0);
            }
            0
        }
    }

    /// Gets the child at the specified index
    ///
    /// # Arguments
    /// * `index` - The index of the child to retrieve
    ///
    /// # Returns
    /// * `Result<Transform, String>` - The child Transform
    pub fn get_child(&self, index: i32) -> Result<Transform, String> {
        unsafe {
            let obj = Object::from_ptr(self.as_ptr());
            let mut params = [&index as *const i32 as *mut c_void];
            let ptr = obj
                .method(("GetChild", ["System.Int32"]))
                .ok_or("Method 'GetChild' not found")?
                .call::<*mut c_void>(&mut params)?;

            if ptr.is_null() {
                return Err("Child is null".to_string());
            }

            Ok(Transform::from_ptr(ptr))
        }
    }
}
