use std::{
    alloc::{self, Layout, LayoutError},
    ptr::NonNull,
};

const DEFAULT_ARRAY_SIZE: usize = 8;

pub struct RawArray<T: Sized> {
    cap: usize,
    ptr: NonNull<T>,
}

impl<T: Sized> Clone for RawArray<T> {
    fn clone(&self) -> Self {
        RawArray {
            cap: self.cap,
            ptr: self.ptr,
        }
    }
}

impl<T: Sized> Copy for RawArray<T> {}

impl<T: Sized> RawArray<T> {
    pub fn new() -> RawArray<T> {
        RawArray {
            cap: 0,
            ptr: NonNull::dangling(),
        }
    }

    pub fn with_capacity(cap: usize) -> Result<RawArray<T>, LayoutError> {
        let layout = Layout::array::<T>(cap)?;
        let new_ptr: *mut u8;
        unsafe {
            new_ptr = alloc::alloc(layout);
        }
        if new_ptr.is_null() {
            alloc::handle_alloc_error(layout)
        } else {
            Ok(RawArray {
                cap,
                ptr: NonNull::new(new_ptr as *mut T).unwrap(),
            })
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn resize(&mut self, new_cap: usize) -> Result<(), LayoutError> {
        if new_cap == 0 {
            self.cap = 0;
            self.ptr = NonNull::dangling();
        } else if self.ptr == NonNull::dangling() {
            *self = Self::with_capacity(new_cap)?;
        } else {
            let old_ptr = self.as_ptr() as *mut T;
            let old_layout = Layout::array::<T>(self.cap)?;

            let new_layout = Layout::array::<T>(new_cap)?;
            let new_ptr: *mut T;
            unsafe {
                new_ptr =
                    alloc::realloc(old_ptr as *mut u8, old_layout, new_layout.size()) as *mut T;
            }

            self.ptr = NonNull::new(new_ptr).unwrap();
            self.cap = new_cap;
        }
        Ok(())
    }

    pub fn grow(&mut self) -> Result<usize, LayoutError> {
        if self.cap == 0 {
            self.resize(DEFAULT_ARRAY_SIZE)?;
            Ok(DEFAULT_ARRAY_SIZE)
        } else {
            let new_cap = self.cap.checked_add(self.cap / 2).unwrap_or(usize::MAX);
            self.resize(new_cap)?;
            Ok(new_cap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_array() {
        let mut raw_array_1: RawArray<usize> = RawArray::new();
        assert_eq!(raw_array_1.cap, 0);
        assert_eq!(raw_array_1.ptr, NonNull::dangling());

        let mut raw_array_2: RawArray<usize> = RawArray::with_capacity(1).unwrap();
        assert_eq!(raw_array_2.capacity(), 1);
        assert_ne!(raw_array_2.ptr, NonNull::dangling());

        raw_array_2.resize(2).unwrap();
        assert_eq!(raw_array_2.capacity(), 2);
        assert_ne!(raw_array_2.ptr, NonNull::dangling());

        raw_array_2.grow().unwrap();
        assert_eq!(raw_array_2.capacity(), 3);

        raw_array_2.grow().unwrap();
        assert_eq!(raw_array_2.capacity(), 4);

        raw_array_2.grow().unwrap();
        assert_eq!(raw_array_2.capacity(), 6);

        raw_array_1.grow().unwrap();
        assert_eq!(raw_array_1.capacity(), 8);
    }
}
