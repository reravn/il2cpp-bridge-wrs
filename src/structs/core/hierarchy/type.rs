//! Type metadata wrapper.
use std::ffi::c_void;

/// Represents a hydrated IL2CPP type.
///
/// `Type` values usually come from fields, properties, method arguments, and
/// return metadata. They are useful for comparison, reflection-style queries,
/// and producing readable dump output.
#[derive(Debug, Clone)]
pub struct Type {
    /// Pointer to the internal IL2CPP type structure
    pub address: *mut c_void,
    /// Name of the type
    pub name: String,
    /// Size of the type in bytes (if applicable)
    pub size: i32,
}

impl Default for Type {
    fn default() -> Self {
        Self {
            address: std::ptr::null_mut(),
            name: String::new(),
            size: 0,
        }
    }
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl Type {
    /// Returns the managed `System.Type` object for this type.
    pub fn get_object(&self) -> *mut c_void {
        unsafe { crate::api::type_get_object(self.address) }
    }

    /// Returns the raw `Il2CppClass*` for this type.
    pub fn get_class(&self) -> *mut c_void {
        unsafe { crate::api::class_from_type(self.address) }
    }

    /// Returns `true` when the two IL2CPP types are equal.
    pub fn equals(&self, other: &Type) -> bool {
        unsafe { crate::api::type_equals(self.address, other.address) }
    }

    /// Returns `true` if this type is passed by reference.
    pub fn is_byref(&self) -> bool {
        unsafe { crate::api::type_is_byref(self.address) }
    }

    /// Returns the underlying IL2CPP type enum value.
    pub fn get_type_enum(&self) -> i32 {
        unsafe { crate::api::type_get_type(self.address) }
    }

    /// Returns a readable C#-style display name for this type.
    ///
    /// Uses `class_from_type` to get the authoritative class name, then handles
    /// generic types by replacing type arguments with `T0, T1, ...` placeholders.
    /// Falls back to parsing `self.name` directly if the class pointer is unavailable.
    ///
    /// # Returns
    /// * `String` - The simplified C# style type name (e.g. `List<T0>`, `int`, `void`)
    pub fn cpp_name(&self) -> String {
        if !self.name.is_empty() {
            let from_type_name = Self::format_type_name_str(&self.name);
            // Keep concrete generic arguments from type_get_name when available.
            if from_type_name.contains('<') {
                return from_type_name;
            }
        }

        if !self.address.is_null() {
            unsafe {
                let class_ptr = crate::api::class_from_type(self.address);
                if !class_ptr.is_null() {
                    let name_ptr = crate::api::class_get_name(class_ptr);
                    if !name_ptr.is_null() {
                        let raw = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy();
                        return Self::format_class_name(&raw);
                    }
                }
            }
        }
        // Fallback: parse the stored type name string directly.
        Self::format_type_name_str(&self.name)
    }

    /// Formats a raw IL2CPP class name (as returned by `class_get_name`) into a
    /// clean C# display name.
    ///
    /// - Generic types (`List`1`, `Dictionary`2`) become `List<T0>`, `Dictionary<T0, T1>`
    /// - `Void` → `void`, `Boolean` → `bool`
    /// - Everything else is returned as-is (already namespace-stripped by IL2CPP)
    fn format_class_name(raw: &str) -> String {
        if let Some(alias) = Self::csharp_alias(raw) {
            return alias.to_string();
        }

        if let Some(element) = raw.strip_suffix("[]") {
            let mapped = Self::csharp_alias(element).unwrap_or(element);
            return format!("{mapped}[]");
        }

        if let Some(bt_pos) = raw.find('`') {
            let base = &raw[..bt_pos];
            let after = &raw[bt_pos + 1..];
            let arity: usize = after
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            if arity > 0 {
                let params: Vec<String> = (0..arity).map(|i| format!("T{}", i)).collect();
                return format!("{}<{}>", base, params.join(", "));
            }
            return base.to_string();
        }
        raw.to_string()
    }

    /// Fallback formatter that parses the raw `type_get_name` string directly.
    /// Handles both backtick notation (`List`1[...]`) and angle-bracket notation
    /// (`List<...>`).
    fn format_type_name_str(name: &str) -> String {
        let name = name.trim();

        if let Some(alias) = Self::csharp_alias(name) {
            return alias.to_string();
        }

        if let Some(element) = name.strip_suffix("[]") {
            return format!("{}[]", Self::format_type_name_str(element));
        }

        if let Some((base, generic_args)) = Self::split_generic_type(name) {
            let simple = base.rfind('.').map(|p| &base[p + 1..]).unwrap_or(base);
            if generic_args.is_empty() {
                let arity = Self::generic_arity(name);
                if arity > 0 {
                    let params: Vec<String> = (0..arity).map(|i| format!("T{}", i)).collect();
                    return format!("{}<{}>", simple, params.join(", "));
                }
                return simple.to_string();
            }

            let rendered_args = generic_args
                .iter()
                .map(|arg| Self::format_type_name_str(arg))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("{simple}<{rendered_args}>");
        }
        let simple = name.rfind('.').map(|p| &name[p + 1..]).unwrap_or(name);
        if let Some(alias) = Self::csharp_alias(simple) {
            alias.to_string()
        } else {
            simple.to_string()
        }
    }

    fn generic_arity(name: &str) -> usize {
        if let Some(bt_pos) = name.find('`') {
            return name[bt_pos + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
        }
        0
    }

    fn split_generic_type(name: &str) -> Option<(&str, Vec<&str>)> {
        let bt_pos = name.find('`')?;
        let base = &name[..bt_pos];
        let after = &name[bt_pos + 1..];
        let digit_count = after.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count == 0 {
            return None;
        }

        let rest = after[digit_count..].trim();
        if rest.is_empty() {
            return Some((base, Vec::new()));
        }

        if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            return Some((base, Self::split_top_level(inner)));
        }
        if let Some(inner) = rest.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
            return Some((base, Self::split_top_level(inner)));
        }
        Some((base, Vec::new()))
    }

    fn split_top_level(s: &str) -> Vec<&str> {
        let mut parts = Vec::new();
        let mut start = 0usize;
        let mut angle_depth = 0i32;
        let mut square_depth = 0i32;

        for (i, c) in s.char_indices() {
            match c {
                '<' => angle_depth += 1,
                '>' => angle_depth -= 1,
                '[' => square_depth += 1,
                ']' => square_depth -= 1,
                ',' if angle_depth == 0 && square_depth == 0 => {
                    parts.push(s[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }

        if start <= s.len() {
            let tail = s[start..].trim();
            if !tail.is_empty() {
                parts.push(tail);
            }
        }

        parts
    }

    fn csharp_alias(name: &str) -> Option<&'static str> {
        Some(match name {
            "System.Void" | "Void" => "void",
            "System.Boolean" | "Boolean" => "bool",
            "System.Byte" | "Byte" => "byte",
            "System.SByte" | "SByte" => "sbyte",
            "System.Int16" | "Int16" => "short",
            "System.UInt16" | "UInt16" => "ushort",
            "System.Int32" | "Int32" => "int",
            "System.UInt32" | "UInt32" => "uint",
            "System.Int64" | "Int64" => "long",
            "System.UInt64" | "UInt64" => "ulong",
            "System.Single" | "Single" => "float",
            "System.Double" | "Double" => "double",
            "System.Decimal" | "Decimal" => "decimal",
            "System.Char" | "Char" => "char",
            "System.String" | "String" => "string",
            "System.Object" | "Object" => "object",
            _ => return None,
        })
    }
}
