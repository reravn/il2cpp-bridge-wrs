//! Unity Component wrapper and trait
use super::game_object::GameObject;
use super::transform::Transform;
use super::unity_object::UnityObject;
use crate::structs::core::{Class, Il2cppObject};
use std::ffi::c_void;
use std::ops::Deref;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Component {
    /// Base UnityObject structure
    pub object: UnityObject,
    /// Cached pointer to the component
    pub m_cached_ptr: *mut c_void,
}

pub trait ComponentTrait {
    fn from_ptr(ptr: *mut c_void) -> Self;
}

impl ComponentTrait for Component {
    fn from_ptr(ptr: *mut c_void) -> Self {
        let object = UnityObject::from_ptr(ptr);
        let m_cached_ptr = unsafe {
            let offset = std::mem::size_of::<Il2cppObject>() as isize;
            *(ptr.offset(offset) as *mut *mut c_void)
        };
        Self {
            object,
            m_cached_ptr,
        }
    }
}

impl Component {
    /// Creates a Component from a raw pointer using the trait implementation
    ///
    /// # Arguments
    /// * `ptr` - The raw pointer to the component
    ///
    /// # Returns
    /// * `Self` - The created Component wrapper
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        <Self as ComponentTrait>::from_ptr(ptr)
    }

    /// Returns the raw pointer to the component
    ///
    /// # Returns
    /// * `*mut c_void` - The raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.object.as_ptr()
    }

    /// Gets the GameObject associated with this component
    ///
    /// # Returns
    /// * `Result<GameObject, String>` - The GameObject attached to this component
    pub fn get_game_object(&self) -> Result<GameObject, String> {
        unsafe {
            let ptr = self
                .method("get_gameObject")
                .ok_or("Method 'get_gameObject' not found")?
                .call::<*mut c_void>(&[])?;

            if ptr.is_null() {
                return Err("Component.gameObject is null".to_string());
            }

            Ok(GameObject::from_ptr(ptr))
        }
    }

    /// Gets the Transform attached to this component
    ///
    /// # Returns
    /// * `Result<Transform, String>` - The Transform attached to this component
    pub fn get_transform(&self) -> Result<Transform, String> {
        unsafe {
            let ptr = self
                .method("get_transform")
                .ok_or("Method 'get_transform' not found")?
                .call::<*mut c_void>(&[])?;

            if ptr.is_null() {
                return Err("Component.transform is null".to_string());
            }

            Ok(Transform::from_ptr(ptr))
        }
    }

    /// Gets a component of the specified class on the same GameObject
    ///
    /// # Type Parameters
    /// * `T` - The type of component to retrieve (must implement ComponentTrait)
    ///
    /// # Arguments
    /// * `class` - The IL2CPP class of the component
    ///
    /// # Returns
    /// * `Result<T, String>` - The requested component, or error if not found
    pub fn get_component<T: ComponentTrait>(&self, class: &Class) -> Result<T, String> {
        if class.object.is_null() {
            return Err(format!(
                "Class '{}' does not have a valid System.Type object",
                class.name
            ));
        }

        unsafe {
            let ptr = self
                .method(("GetComponent", ["System.Type"]))
                .ok_or("Method 'GetComponent' not found")?
                .call::<*mut c_void>(&[class.object as *mut c_void])?;

            if ptr.is_null() {
                return Err("Component not found".to_string());
            }

            Ok(T::from_ptr(ptr))
        }
    }
}

impl Deref for Component {
    type Target = UnityObject;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}
