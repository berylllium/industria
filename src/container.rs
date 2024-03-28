use std::{
    mem,
    ptr::{self, NonNull},
    alloc::{self, Layout},
};

pub struct FreeList<T> {
    cap: usize,
    data: NonNull<T>,
    free_indices: NonNull<bool>,
}

impl<T> FreeList<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "FreeList does not allow ZSTs.");

        Self {
            cap: 0,
            data: NonNull::dangling(),
            free_indices: NonNull::dangling(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        assert!(mem::size_of::<T>() != 0, "FreeList does not allow ZSTs.");

        let data = {
            let layout = Layout::array::<T>(cap).unwrap();
            let data = unsafe { alloc::alloc(layout) };

            match NonNull::new(data as *mut T) {
                Some(p) => p,
                None => alloc::handle_alloc_error(layout),
            }
        };

        let free_indices = {
            let layout = Layout::array::<bool>(cap).unwrap();
            let data = unsafe { alloc::alloc(layout) };

            match NonNull::new(data as *mut bool) {
                Some(p) => p,
                None => alloc::handle_alloc_error(layout),
            }
        };

        // Set indices to be free.
        for i in 0..cap {
            unsafe { ptr::write(free_indices.as_ptr().add(i), true) };
        }

        Self {
            cap,
            data,
            free_indices,
        }
    }
}

impl<T> FreeList<T> {
    pub fn push_first(&mut self, value: T) -> usize {
        let insert_index = match self.find_empty_index() {
            Some(p) => p,
            None => { self.grow(); self.find_empty_index().unwrap() },
        };

        unsafe {
            ptr::write(self.data.as_ptr().add(insert_index), value);
            ptr::write(self.free_indices.as_ptr().add(insert_index), true);
        }

        insert_index
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.cap) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.cap) }
    }
}

impl<T> FreeList<T> {
    fn grow(&mut self) {
        let (new_cap, new_data_layout, new_free_indices_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap(), Layout::array::<bool>(1).unwrap())
        } else if self.cap == 1 {
            (2, Layout::array::<T>(2).unwrap(), Layout::array::<bool>(2).unwrap())
        } else {
            let new_cap = (1.5f32 * self.cap as f32) as usize;

            (new_cap, Layout::array::<T>(new_cap).unwrap(), Layout::array::<bool>(new_cap).unwrap())
        };

        assert!(new_data_layout.size() <= isize::MAX as usize, "Allocation too large.");

        self.data = {
            let new_ptr = if self.cap == 0 {
                unsafe { alloc::alloc(new_data_layout) }
            } else {
                let old_layout = Layout::array::<T>(self.cap).unwrap();

                let old_ptr = self.data.as_ptr() as *mut u8;
                unsafe { alloc::realloc(old_ptr, old_layout, new_data_layout.size()) }
            };

            match NonNull::new(new_ptr as *mut T) {
                Some(p) => p,
                None => alloc::handle_alloc_error(new_data_layout),
            }
        };

        self.free_indices = {
            let new_ptr = if self.cap == 0 {
                unsafe { alloc::alloc(new_free_indices_layout) }
            } else {
                let old_layout = Layout::array::<bool>(self.cap).unwrap();

                let old_ptr = self.free_indices.as_ptr() as *mut u8;
                unsafe { alloc::realloc(old_ptr, old_layout, new_free_indices_layout.size()) }
            };

            match NonNull::new(new_ptr as *mut bool) {
                Some(p) => p,
                None => alloc::handle_alloc_error(new_free_indices_layout),
            }
        };

        for i in self.cap..new_cap {
            unsafe { ptr::write(self.free_indices.as_ptr().add(i), true) };
        }
    }

    fn find_empty_index(&self) -> Option<usize> {
        for i in 0..self.cap {
            if unsafe { *self.free_indices.as_ptr().add(i) } {
                return Some(i);
            }
        }

        None
    }
}

impl<T> Drop for FreeList<T> {
    fn drop(&mut self) {
        if self.cap == 0 { return; }

        for i in 0..self.cap {
            unsafe {
                if *self.free_indices.as_ptr().add(i)  {
                    ptr::drop_in_place(self.data.as_ptr().add(i));
                }
            }
        }

        unsafe {
            alloc::dealloc(self.data.as_ptr() as *mut u8, Layout::array::<T>(self.cap).unwrap());
            alloc::dealloc(self.free_indices.as_ptr() as *mut u8, Layout::array::<bool>(self.cap).unwrap());
        }
    }
}
