//! Symbol resolution and caching utilities

use dashmap::DashMap;
use once_cell::sync::Lazy;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can occur during symbol resolution
pub enum SymbolError {
    /// The specified symbol was not found
    #[error("Symbol not found: {0}")]
    NotFound(String),
    /// Failed to convert the symbol name to a CString
    #[error("CString error")]
    StringError,
}

static CACHE: Lazy<DashMap<String, usize>> = Lazy::new(|| DashMap::new());

/// Resolves a symbol to its address using platform-specific lookup.
///
/// # Arguments
/// * `symbol` - The name of the symbol to resolve (e.g., "MGCopyAnswer")
///
/// # Returns
/// * `Result<usize, SymbolError>` - The address of the symbol or an error
pub fn resolve_symbol(symbol: &str) -> Result<usize, SymbolError> {
    if let Some(entry) = CACHE.get(symbol) {
        return Ok(*entry);
    }

    let addr = platform::raw_resolve(symbol)?;
    CACHE.insert(symbol.into(), addr);
    Ok(addr)
}

/// Manually caches a symbol address
///
/// Use this if you have resolved a symbol via other means and want to store it for future lookups.
///
/// # Arguments
/// * `s` - The symbol name
/// * `a` - The symbol address
pub fn cache_symbol(s: &str, a: usize) {
    CACHE.insert(s.into(), a);
}

/// Clears the symbol cache
pub fn clear_cache() {
    CACHE.clear();
}

// macOS / iOS: use dlsym with RTLD_DEFAULT to search all loaded dylibs.
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod platform {
    use super::SymbolError;
    use std::ffi::CString;

    pub fn raw_resolve(symbol: &str) -> Result<usize, SymbolError> {
        let c_str = CString::new(symbol).map_err(|_| SymbolError::StringError)?;
        unsafe {
            let addr_ptr = libc::dlsym(libc::RTLD_DEFAULT, c_str.as_ptr());
            if addr_ptr.is_null() {
                Err(SymbolError::NotFound(symbol.into()))
            } else {
                Ok(addr_ptr as usize)
            }
        }
    }
}

// Linux / Android: use dlsym with RTLD_DEFAULT to search all loaded shared objects.
#[cfg(any(target_os = "linux", target_os = "android"))]
mod platform {
    use super::SymbolError;
    use std::ffi::CString;

    pub fn raw_resolve(symbol: &str) -> Result<usize, SymbolError> {
        let c_str = CString::new(symbol).map_err(|_| SymbolError::StringError)?;
        unsafe {
            let addr_ptr = libc::dlsym(libc::RTLD_DEFAULT, c_str.as_ptr());
            if addr_ptr.is_null() {
                Err(SymbolError::NotFound(symbol.into()))
            } else {
                Ok(addr_ptr as usize)
            }
        }
    }
}

// Windows: use GetProcAddress with a null module handle to search the main executable.
#[cfg(target_os = "windows")]
mod platform {
    use super::SymbolError;
    use std::ffi::CString;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    pub fn raw_resolve(symbol: &str) -> Result<usize, SymbolError> {
        let c_str = CString::new(symbol).map_err(|_| SymbolError::StringError)?;
        unsafe {
            // Null handle returns the calling process module, similar to RTLD_DEFAULT on Unix.
            let handle = GetModuleHandleA(std::ptr::null());
            if handle == 0 {
                return Err(SymbolError::NotFound(symbol.into()));
            }
            let addr = GetProcAddress(handle, c_str.as_ptr() as *const u8);
            match addr {
                Some(f) => Ok(f as usize),
                None => Err(SymbolError::NotFound(symbol.into())),
            }
        }
    }
}

// Unsupported platforms: always return an error.
#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "linux",
    target_os = "android",
    target_os = "windows",
)))]
mod platform {
    use super::SymbolError;

    pub fn raw_resolve(symbol: &str) -> Result<usize, SymbolError> {
        Err(SymbolError::NotFound(format!(
            "{} (unsupported platform)",
            symbol
        )))
    }
}
