//! IL2CPP List definition and operations
use super::array::Il2cppArray;
use std::ffi::c_void;
use std::marker::PhantomData;

#[repr(C)]
pub struct Il2cppList<T: Copy> {
    /// Pointer to the list class
    pub klass: *mut c_void,
    /// Monitor for synchronization
    pub monitor: *mut c_void,
    /// Internal array of items
    pub items: *mut Il2cppArray<T>,
    /// Number of elements in the list
    pub size: i32,
    /// Version of the list
    pub version: i32,
    _phantom: PhantomData<T>,
}

impl<T: Copy> Il2cppList<T> {
    /// Gets the internal items array
    ///
    /// # Returns
    /// * `Option<&Il2cppArray<T>>` - The underlying array, or None if null
    pub fn items_array(&self) -> Option<&Il2cppArray<T>> {
        if self.items.is_null() {
            None
        } else {
            unsafe { Some(&*self.items) }
        }
    }

    /// Gets the internal items array as mutable
    ///
    /// # Returns
    /// * `Option<&mut Il2cppArray<T>>` - The underlying mutable array, or None if null
    pub fn items_array_mut(&mut self) -> Option<&mut Il2cppArray<T>> {
        if self.items.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.items) }
        }
    }

    /// Gets the element at the specified index
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// * `Option<T>` - The element if present and index is within bounds, otherwise None
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.size as usize {
            return None;
        }
        self.items_array().map(|arr| arr.get(index))
    }

    /// Alias for `get` that panics on out of bounds
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// * `T` - The element at the specified index
    ///
    /// # Panics
    /// Panics if the index is out of bounds or the item array is null.
    pub fn at(&self, index: usize) -> T {
        self.get(index).expect("Index out of bounds")
    }

    /// Sets the element at the specified index
    ///
    /// # Arguments
    /// * `index` - The index where the value should be set
    /// * `value` - The value to set
    ///
    /// # Returns
    /// * `bool` - True if setting was successful, False if index out of bounds or array null
    pub fn set(&mut self, index: usize, value: T) -> bool {
        if index >= self.size as usize {
            return false;
        }
        if let Some(arr) = self.items_array_mut() {
            arr.set(index, value);
            true
        } else {
            false
        }
    }

    /// Converts the list to a Rust Vec
    ///
    /// # Returns
    /// * `Vec<T>` - A new vector containing the list elements
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.size as usize);
        for i in 0..self.size as usize {
            if let Some(item) = self.get(i) {
                result.push(item);
            }
        }
        result
    }

    /// Gets a pointer to the data array
    ///
    /// # Returns
    /// * `Option<*const T>` - Pointer to the first element, or None
    pub fn get_pointer(&self) -> Option<*const T> {
        self.items_array().map(|arr| arr.get_pointer())
    }

    /// Returns an iterator over the list items
    ///
    /// # Returns
    /// * `impl Iterator<Item = T> + '_` - An iterator yielding elements
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.size as usize).filter_map(|i| self.get(i))
    }
}
