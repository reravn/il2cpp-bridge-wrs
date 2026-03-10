//! IL2CPP Array definition and operations
use crate::api;
use crate::structs::core::Class;
use std::ffi::c_void;
use std::marker::PhantomData;

#[repr(C)]
pub struct ArrayBounds {
    /// Length of the dimension
    pub length: usize,
    /// Lower bound of the dimension
    pub lower_bound: i32,
}

#[repr(C)]
pub struct Il2cppArray<T> {
    /// Pointer to the array class
    pub klass: *mut c_void,
    /// Monitor for synchronization
    pub monitor: *mut c_void,
    /// Pointer to array bounds
    pub bounds: *mut ArrayBounds,
    /// Maximum length of the array
    pub max_length: usize,
    phantom: PhantomData<T>,
}

impl<T: Copy> Il2cppArray<T> {
    /// Gets the address of the data array
    ///
    /// # Returns
    /// * `usize` - The memory address where the array data begins
    pub fn get_data(&self) -> usize {
        let header_size = std::mem::size_of::<Self>();
        (self as *const Self as usize) + header_size
    }

    /// Gets the element at the specified index
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// * `T` - The element at the specified index
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn get(&self, index: usize) -> T {
        if index >= self.max_length {
            panic!("Index out of bounds: {} >= {}", index, self.max_length);
        }

        unsafe {
            let element_ptr = (self.get_data() + index * std::mem::size_of::<T>()) as *const T;
            *element_ptr
        }
    }

    /// Alias for `get`
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// * `T` - The element at the specified index
    pub fn at(&self, index: usize) -> T {
        self.get(index)
    }

    /// Sets the element at the specified index
    ///
    /// # Arguments
    /// * `index` - The index where the value should be set
    /// * `value` - The value to set
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.max_length {
            panic!("Index out of bounds: {} >= {}", index, self.max_length);
        }

        unsafe {
            let element_ptr = (self.get_data() + index * std::mem::size_of::<T>()) as *mut T;
            *element_ptr = value;
        }
    }

    /// Gets a raw pointer to the data
    ///
    /// # Returns
    /// * `*const T` - Pointer to the first element of the array
    pub fn get_pointer(&self) -> *const T {
        let header_size = std::mem::size_of::<Self>();
        ((self as *const Self as usize) + header_size) as *const T
    }

    /// Inserts elements from a slice into the array
    ///
    /// # Arguments
    /// * `arr` - The slice of elements to insert
    /// * `size` - The number of elements to insert
    /// * `index` - The starting index in the array
    pub fn insert(&mut self, arr: &[T], size: usize, index: usize) {
        if (size + index) >= self.max_length {
            if index >= self.max_length {
                return;
            }

            let new_size = self.max_length - index;
            for (i, &item) in arr.iter().enumerate().take(new_size) {
                self.set(i + index, item);
            }
        } else {
            for (i, &item) in arr.iter().enumerate().take(size) {
                self.set(i + index, item);
            }
        }
    }

    /// Fills the array with a value
    ///
    /// # Arguments
    /// * `value` - The value to fill the array with
    pub fn fill(&mut self, value: T) {
        for i in 0..self.max_length {
            self.set(i, value);
        }
    }

    /// Removes an element at the specified index
    ///
    /// Shifts subsequent elements to the left in place.
    ///
    /// IL2CPP arrays are fixed-size, so this does not change `max_length`.
    /// The final slot is left as a duplicate of the former tail element.
    ///
    /// # Arguments
    /// * `index` - The index of the element to remove
    pub fn remove_at(&mut self, index: usize) {
        if index >= self.max_length {
            return;
        }

        if index + 1 < self.max_length {
            unsafe {
                let data = self.get_pointer() as *mut T;
                std::ptr::copy(
                    data.add(index + 1),
                    data.add(index),
                    self.max_length - index - 1,
                );
            }
        }
    }

    /// Removes a range of elements
    ///
    /// Shifts subsequent elements to the left in place.
    ///
    /// IL2CPP arrays are fixed-size, so this does not change `max_length`.
    /// Vacated tail slots are left duplicated from the previous tail values.
    ///
    /// # Arguments
    /// * `index` - The starting index
    /// * `count` - The number of elements to remove
    pub fn remove_range(&mut self, index: usize, mut count: usize) {
        if index >= self.max_length {
            return;
        }

        if count == 0 {
            count = 1;
        }

        let count = count.min(self.max_length - index);
        let tail_len = self.max_length - index - count;

        if tail_len > 0 {
            unsafe {
                let data = self.get_pointer() as *mut T;
                std::ptr::copy(data.add(index + count), data.add(index), tail_len);
            }
        }
    }

    /// Removes all elements from the array
    ///
    /// IL2CPP arrays are fixed-size managed objects, so clearing them by
    /// shrinking the header is unsupported. Overwrite elements explicitly if
    /// you need sentinel values.
    pub fn remove_all(&mut self) {}

    /// Converts the array to a Rust Vec
    ///
    /// # Returns
    /// * `Vec<T>` - A new vector containing the array elements
    pub fn to_vector(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.max_length);
        for i in 0..self.max_length {
            result.push(self.at(i));
        }
        result
    }

    /// Creates a new array instance
    ///
    /// # Arguments
    /// * `class` - The class of the elements
    /// * `size` - The size of the array
    ///
    /// # Returns
    /// * `*mut Self` - A pointer to the new array
    pub fn new(class: &Class, size: usize) -> *mut Self {
        unsafe { api::array_new(class.address, size as u32) as *mut Self }
    }
}
