//! IL2CPP Field definition and functionality
use crate::structs::core::{Class, Type, ValueType};
use crate::api::api;
use std::ffi::c_void;

/// Represents an IL2CPP Field (a variable within a class or struct)
#[derive(Debug, Clone)]
pub struct Field {
    /// Pointer to the internal IL2CPP field structure
    pub address: *mut c_void,
    /// Name of the field
    pub name: String,
    /// Type information of the field
    pub type_info: Type,
    /// Class defining this field
    pub class: Option<*const Class>,
    /// Offset of the field within the instance
    pub offset: i32,
    /// Flags associated with the field
    pub flags: i32,
    /// Whether the field is static
    pub is_static: bool,
    /// Whether the field is constant (literal)
    pub is_literal: bool,
    /// Whether the field is readonly (init only)
    pub is_readonly: bool,
    /// Whether the field is not serialized
    pub is_not_serialized: bool,
    /// Whether the field has a special name
    pub is_special_name: bool,
    /// Whether the field is pinvoke impl
    pub is_pinvoke_impl: bool,
    /// Instance pointer for instance fields
    pub instance: Option<*mut c_void>,
}

unsafe impl Send for Field {}
unsafe impl Sync for Field {}

impl Field {
    /// Generates a string representation of the field, including its value if static
    ///
    /// # Returns
    /// * `String` - The formatted string representation
    pub fn to_string(&self) -> String {
        let access = self.get_attribute();
        let qualifier = if self.is_literal {
            "const ".to_string()
        } else {
            let mut s = String::new();
            if self.is_static {
                s.push_str("static ");
            }
            if self.is_readonly {
                s.push_str("readonly ");
            }
            s
        };

        let value_str: Option<String> = None;
        format!(
            "{} {}{} {}{}; // 0x{:X}",
            access,
            qualifier,
            self.type_info.cpp_name(),
            self.name,
            value_str.unwrap_or_default(),
            self.offset
        )
    }

    /// Reads the value of a field (static or instance)
    ///
    /// # Type Parameters
    /// * `T` - The type to read into. Must be `Copy` and `'static`.
    ///
    /// # Returns
    /// * `Result<T, String>` - The value read, or an error if reading fails
    pub unsafe fn get_value<T: Copy + 'static>(&self) -> Result<T, String> {
        if self.is_static {
            if let Some(class) = self.class {
                if (*class).is_generic {
                    return Err("Cannot read static field of generic class".to_string());
                }
            }

            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<ValueType>() {
                let class = self
                    .class
                    .ok_or_else(|| "Field does not have a parent class reference".to_string())?;

                let static_data = api::class_get_static_field_data(class as *mut _);
                if static_data.is_null() {
                    return Err("Static field data is null".to_string());
                }

                let address = static_data as usize + self.offset as usize;

                let field_type = api::field_get_type(self.address);
                let type_class = api::class_from_type(field_type);

                if type_class.is_null() {
                    return Err("Failed to resolve field type class".to_string());
                }

                let vt = ValueType::from_ptr_with_class(address as *mut c_void, type_class);
                return Ok(std::ptr::read(&vt as *const _ as *const T));
            }

            let mut value: T = std::mem::zeroed();
            api::field_static_get_value(self.address, &mut value as *mut T as *mut c_void);
            Ok(value)
        } else {
            let instance = self.instance.ok_or_else(|| {
                format!(
                    "Field '{}' is an instance field but no instance was provided. Use Object::field or set the instance manually.",
                    self.name
                )
            })?;

            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<ValueType>() {
                let field_type = api::field_get_type(self.address);
                if field_type.is_null() {
                    return Err("Failed to get field type".to_string());
                }
                let type_class = api::class_from_type(field_type);
                if type_class.is_null() {
                    return Err("Failed to resolve class from field type".to_string());
                }

                let address = instance as usize + self.offset as usize;
                let vt = ValueType::from_ptr_with_class(address as *mut c_void, type_class);
                return Ok(std::ptr::read(&vt as *const _ as *const T));
            }

            let address = instance as usize + self.offset as usize;  
            crate::memory::rw::read(address).map_err(|e| e.to_string())
        }
    }

    /// Writes a value to a field (static or instance)
    ///
    /// # Type Parameters
    /// * `T` - The type of value to write. Must be `Copy`.
    ///
    /// # Arguments
    /// * `value` - The value to write to the field
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if success, Err if failure
    pub unsafe fn set_value<T: Copy>(&self, value: T) -> Result<(), String> {
        if self.is_static {
            api::field_static_set_value(self.address, &value as *const T as *mut c_void);
            Ok(())
        } else {
            let instance = self.instance.ok_or_else(|| {
                format!(
                    "Field '{}' is an instance field but no instance was provided. Use Object::field or set the instance manually.",
                    self.name
                )
            })?;

            let address = instance as usize + self.offset as usize;
            crate::memory::rw::write(address, value).map_err(|e| e.to_string())
        }
    }

    /// Gets the access modifier attribute string
    fn get_attribute(&self) -> &'static str {
        match self.flags & api::FIELD_ATTRIBUTE_FIELD_ACCESS_MASK {
            api::FIELD_ATTRIBUTE_PRIVATE => "private",
            api::FIELD_ATTRIBUTE_PUBLIC => "public",
            api::FIELD_ATTRIBUTE_FAMILY => "protected",
            api::FIELD_ATTRIBUTE_ASSEMBLY => "internal",
            api::FIELD_ATTRIBUTE_FAM_AND_ASSEM => "private protected",
            api::FIELD_ATTRIBUTE_FAM_OR_ASSEM => "protected internal", 
            _ => "private",
        }
    }

    /// Gets the parent class of the field
    ///
    /// # Returns
    /// * `*mut c_void` - Pointer to the parent class structure
    pub fn get_parent(&self) -> *mut c_void {
        unsafe { api::field_get_parent(self.address) }
    }
}
