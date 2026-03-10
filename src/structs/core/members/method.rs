//! IL2CPP Method definition and functionality
use crate::structs::collections::Il2cppArray;
use crate::structs::core::{Class, Object, Type};
use crate::api::{self, cache, invoke_method};
use std::ffi::c_void;
use std::ptr;

/// Argument information for a method
#[derive(Debug, Clone)]
pub struct Arg {
    /// Name of the argument
    pub name: String,
    /// Type information of the argument
    pub type_info: Type,
}

/// IL2CPP Method structure definition
#[derive(Debug, Clone)]
pub struct Method {
    /// Pointer to the internal IL2CPP method structure
    pub address: *mut c_void,
    /// Metadata token for this method
    pub token: u32,
    /// Name of the method
    pub name: String,
    /// Class defining this method
    pub class: Option<*const Class>,
    /// Return type of the method
    pub return_type: Type,
    /// Flags associated with the method
    pub flags: i32,
    /// Whether the method is static
    pub is_static: bool,
    /// Pointer to the potentially compiled native function
    pub function: *mut c_void,
    /// Relative Virtual Address of the method (static offset from image base)
    pub rva: u64,
    /// Virtual Address of the method at runtime (includes ASLR slide)
    pub va: u64,
    /// List of arguments for this method
    pub args: Vec<Arg>,
    /// Whether the method is generic
    pub is_generic: bool,
    /// Whether the method is an inflated generic method
    pub is_inflated: bool,
    /// Whether the method is an instance method
    pub is_instance: bool,
    /// Number of parameters this method accepts
    pub param_count: u8,
    /// Pointer to the declaring type
    pub declaring_type: *mut c_void,
    /// Instance pointer for instance methods
    pub instance: Option<*mut c_void>,
}

unsafe impl Send for Method {}
unsafe impl Sync for Method {}

/// Implementation of Method operations
impl Method {
    /// Generates a string representation of the method signature
    ///
    /// # Returns
    /// * `String` - The formatted method signature
    pub fn to_string(&self) -> String {
        let access = self.get_attribute();
        let flags = self.flags;

        let is_abstract = (flags & api::METHOD_ATTRIBUTE_ABSTRACT) != 0;
        let is_virtual = (flags & api::METHOD_ATTRIBUTE_VIRTUAL) != 0;
        let is_final = (flags & api::METHOD_ATTRIBUTE_FINAL) != 0;

        let qualifier = if self.is_static {
            "static "
        } else if is_abstract {
            "abstract "
        } else if is_virtual && !is_final {
            "virtual "
        } else {
            ""
        };

        let args_str = self
            .args
            .iter()
            .map(|arg| format!("{} {}", arg.type_info.cpp_name(), arg.name))
            .collect::<Vec<_>>()
            .join(", ");

        let rva_comment = if self.rva == 0 {
            "// RVA: -1 Offset: -1 VA: -1".to_string()
        } else {
            format!(
                "// RVA: 0x{:X} Offset: 0x{:X} VA: 0x{:X}",
                self.rva, self.rva, self.va
            )
        };

        format!(
            "{}\n{} {}{} {}({}) {{ }}",
            rva_comment,
            access,
            qualifier,
            self.return_type.cpp_name(),
            self.name,
            args_str,
        )
    }

    /// Invokes the method with the provided parameters
    ///
    /// # Type Parameters
    /// * `T` - The expected return type
    ///
    /// # Arguments
    /// * `params` - A slice of raw pointers to the arguments
    ///
    /// # Returns
    /// * `Result<T, String>` - The return value or an error
    pub unsafe fn call<T>(&self, params: &[*mut c_void]) -> Result<T, String> {
        let instance = if self.is_static {
            ptr::null_mut()
        } else {
            match self.instance {
                Some(inst) => inst,
                None => {
                    return Err(format!(
                        "Method '{}' is not static but no instance was provided. Use Object::method or set the instance manually.",
                        self.name
                    ));
                }
            }
        };

        if params.len() != self.args.len() {
            return Err(format!(
                "Argument count mismatch: expected {}, got {}",
                self.args.len(),
                params.len()
            ));
        }

        let params_ptr = if params.is_empty() {
            ptr::null()
        } else {
            params.as_ptr()
        };

        let result = invoke_method(self.address, instance, params_ptr)?;

        let return_class = api::class_from_type(self.return_type.address);
        if return_class.is_null() {
            return Ok(std::mem::transmute_copy(&result));
        }

        if api::class_is_valuetype(return_class) {
            if result.is_null() {
                return Ok(std::mem::zeroed());
            }

            let unboxed = api::object_unbox(result);
            Ok(ptr::read(unboxed as *const T))
        } else {
            Ok(std::mem::transmute_copy(&result))
        }
    }

    /// Gets the access modifier attribute string
    pub fn get_attribute(&self) -> &'static str {
        match self.flags & api::METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK {
            api::METHOD_ATTRIBUTE_PRIVATE => "private",
            api::METHOD_ATTRIBUTE_PUBLIC => "public",
            api::METHOD_ATTRIBUTE_FAMILY => "protected",
            api::METHOD_ATTRIBUTE_ASSEM => "internal",
            api::METHOD_ATTRIBUTE_FAM_AND_ASSEM => "private protected",
            api::METHOD_ATTRIBUTE_FAM_OR_ASSEM => "protected internal",
            _ => "private",
        }
    }

    /// Gets the method object
    ///
    /// # Returns
    /// * `*mut c_void` - The pointer to the MethodInfo object
    pub fn get_object(&self) -> *mut c_void {
        unsafe { api::method_get_object(self.address, ptr::null_mut()) }
    }

    /// Inflates a generic method with the specified type arguments to create a concrete generic method.
    ///
    /// # Arguments
    /// * `classes` - The type arguments
    ///
    /// # Returns
    /// * `Result<Method, String>` - The inflated method
    pub fn inflate(&self, classes: &[&Class]) -> Result<Method, String> {
        unsafe {
            if !self.is_generic {
                return Err(format!(
                    "Method '{}' is not a generic method definition",
                    self.name
                ));
            }

            if classes.is_empty() {
                return Err("No type arguments provided".to_string());
            }

            let mut type_args = Vec::with_capacity(classes.len());
            for (i, cls) in classes.iter().enumerate() {
                if cls.object.is_null() {
                    return Err(format!(
                        "Class '{}' (arg {}) has no Type object",
                        cls.name, i
                    ));
                }
                type_args.push(cls.object);
            }

            let method_object = self.get_object();
            if method_object.is_null() {
                return Err(format!(
                    "Could not get MethodInfo object for method '{}'",
                    self.name
                ));
            }

            let method_obj = Object::from_ptr(method_object);

            let corlib = cache::mscorlib();
            let type_class = corlib
                .class("System.Type")
                .ok_or_else(|| "Could not find System.Type class".to_string())?;

            let type_array = Il2cppArray::<*mut c_void>::new(&type_class, type_args.len());
            if type_array.is_null() {
                return Err("Could not create Type[] array".to_string());
            }

            let array_ref = &mut *type_array;
            for (i, &type_arg) in type_args.iter().enumerate() {
                array_ref.set(i, type_arg);
            }

            let make_generic_method = method_obj
                .method(("MakeGenericMethod", ["System.Type[]"]))
                .ok_or_else(|| "Could not find MakeGenericMethod(Type[]) method".to_string())?;

            let inflated_method_obj =
                make_generic_method.call::<*mut c_void>(&[type_array as *mut c_void])?;

            if inflated_method_obj.is_null() {
                return Err("MakeGenericMethod returned null".to_string());
            }

            let inflated_obj = Object::from_ptr(inflated_method_obj);

            let mhandle_field = inflated_obj
                .field("mhandle")
                .unwrap()
                .get_value::<*mut c_void>()
                .map_err(|e| format!("Could not read mhandle field: {}", e))?;

            if mhandle_field.is_null() {
                return Err("mhandle field is null".to_string());
            }

            cache::method_from_ptr(mhandle_field).ok_or_else(|| {
                "Could not convert inflated method pointer to Method struct".to_string()
            })
        }
    }
}
