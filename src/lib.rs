//! A extremely small and simple vector datastructure.
//!
//! This is a vector, which is heap allocated and grows dynamically.
//! It does not serve as a drop-in replacement of `std::vec` but
//! as an easy to use and small alternative.

//! # Examples
//!
//! ```
//! use minivec::MiniVec;
//!
//! fn main () {
//!     let mut vector = MiniVec::new();
//!
//!     vector.push(3);
//!     assert_eq!(vector.size(), 1);
//!     assert_eq!(vector[0], 3);
//!
//!     vector.remove(0);
//!     assert_eq!(vector.size(), 0);
//! }
//! ```
use std::alloc;
use std::ops::Index;
use std::ops::IndexMut;

pub struct MiniVec<T: Clone> {
    capacity: usize,
    size: usize,
    buffer: Option<*mut T>,
}

impl<T: Clone> Drop for MiniVec<T> {
    fn drop(&mut self) {
        if let Some(v) = self.buffer {
            let layout = alloc::Layout::array::<T>(self.capacity).unwrap();
            unsafe { alloc::dealloc(v as *mut u8, layout) }
        }
    }
}

// TODO: Add more linting/formatting (https://dev.to/bampeers/rust-ci-with-github-actions-1ne9)
// TODO: find idiomatic way of adding todos
// TODO: create branches: dev, release and the feature branches
// TODO: generate the README from code? (or make it more fancy)
// TODO: add test-coverage to the README
// TODO: add integration-tests
// TODO: figure out how to do memory-leakage testing (valgrind?)

impl<T: Clone> MiniVec<T> {
    const DEFAULT_GROWTH: usize = 2;
    const DEFAULT_FIRST_CAP: usize = 2;

    /// creates a new instance of `MiniVec`.
    ///
    /// ```
    /// use minivec::MiniVec;
    ///
    /// let vector = MiniVec::<usize>::new();
    /// ```
    ///
    /// Upon creation, the vector will have no heap allocation associated with it.
    /// The vector only allocates memory if it actually holds elements.
    pub fn new() -> Self {
        Self {
            capacity: 0,
            size: 0,
            buffer: None,
        }
    }

    /// returns number of elements in this collection.
    ///
    /// ```
    /// use minivec::MiniVec;
    ///
    /// let mut vector = MiniVec::new();
    ///
    /// vector.push(1);
    /// vector.push(2);
    /// vector.push(3);
    ///
    /// assert_eq!(vector.size(), 3);
    /// ```
    pub fn size(&self) -> usize {
        self.size
    }

    /// returns the capacity of the vector.
    ///
    /// ```
    /// use minivec::MiniVec;
    ///
    /// let mut vector = MiniVec::new();
    ///
    /// assert_eq!(vector.capacity(), 0);
    /// vector.push(1);
    /// assert_eq!(vector.capacity(), 2);
    /// vector.push(2);
    /// vector.push(3);
    /// assert_eq!(vector.capacity(), 4);
    /// ```
    /// The capacity is the max. number of elements that can fit
    /// in the currently allocated buffer.
    ///
    /// It grows exponentially by the factor of 2, starts at 0 (no allocations)
    /// and increases at the first allocation to 2.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn grow(&mut self) {
        match self.buffer {
            Some(v) => {
                let layout = alloc::Layout::array::<T>(self.capacity).unwrap();
                self.capacity *= MiniVec::<T>::DEFAULT_GROWTH;
                self.buffer = Some(unsafe {
                    alloc::realloc(
                        v as *mut u8,
                        layout,
                        self.capacity * std::mem::size_of::<T>(),
                    ) as *mut T
                });
                // TODO: check for self.buffer to be null here...
            }
            None => {
                self.capacity = MiniVec::<T>::DEFAULT_FIRST_CAP;
                let layout = alloc::Layout::array::<T>(self.capacity).unwrap();
                self.buffer = Some(unsafe { alloc::alloc(layout) as *mut T });
            }
        }
    }

    /// Add an element to the end of the vector.
    ///
    /// ```
    /// use minivec::MiniVec;
    ///
    /// let mut vector = MiniVec::new();
    /// vector.push(0);
    /// assert_eq!(vector[0], 0);
    /// ```
    ///
    /// This method is O(1). If the size of the vector is the capacity,
    /// it will reallocate the buffer.
    pub fn push(&mut self, new_element: T) {
        if self.size == self.capacity {
            self.grow();
        }

        // TODO: is already checked in grow... unnecessary (maybe use NonNull pointer?)
        unsafe {
            *(self.buffer.unwrap().offset(self.size as isize)) = new_element;
        }
        self.size += 1;
    }

    /// Remove a element on the given position, returns it.
    ///
    /// ```
    /// use minivec::MiniVec;
    ///
    /// let mut vector = MiniVec::new();
    /// vector.push(0);
    /// assert_eq!(vector.remove(0), 0);
    /// ```
    /// This has a worst-case runtime of O(n).
    pub fn remove(&mut self, index: usize) -> T {
        let removed = unsafe { (*(self.buffer.unwrap().offset(index as isize))).clone() };

        unsafe {
            let mut start = self.buffer.unwrap().offset(index as isize);

            for _ in index..self.size - 1 {
                *start = (*(start.offset(1) as *mut T)).clone();
                start = start.offset(1);
            }
        }

        self.size -= 1;

        removed
    }
}

impl<T: Clone> Index<usize> for MiniVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.size {
            panic!("Error: Index too big for vector");
        }

        unsafe { &*(self.buffer.unwrap().offset(index as isize)) }
    }
}

impl<T: Clone> IndexMut<usize> for MiniVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.size {
            panic!("Error: Index too big for vector");
        }

        unsafe { &mut *(self.buffer.unwrap().offset(index as isize)) }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new() {
        let vector = MiniVec::<i32>::new();
        assert_eq!(vector.size(), 0);
        assert_eq!(vector.capacity(), 0);
    }

    #[test]
    fn add_element() {
        let mut vector = MiniVec::new();

        vector.push(1);
        assert_eq!(vector.size(), 1);

        vector.push(2);
        assert_eq!(vector.size(), 2);

        vector.push(3);
        assert_eq!(vector.size(), 3);
    }

    #[test]
    fn get_element_valid() {
        let mut vector = MiniVec::new();

        vector.push(1);
        vector.push(2);
        vector.push(3);

        assert_eq!(vector[0], 1);
        assert_eq!(vector[1], 2);
        assert_eq!(vector[2], 3);
    }

    #[test]
    #[should_panic]
    fn get_element_invalid() {
        let mut vector = MiniVec::new();

        vector.push(1);
        vector.push(2);
        vector.push(3);

        vector[100];
    }

    #[test]
    fn remove_element_by_index_valid() {
        let mut vector = MiniVec::new();

        vector.push(1);
        vector.push(2);
        vector.push(3);

        vector.remove(0);
        assert_eq!(vector.size(), 2);
        assert_eq!(vector[0], 2);
        assert_eq!(vector[1], 3);

        assert_eq!(vector.remove(1), 3);
        assert_eq!(vector.size(), 1);
        assert_eq!(vector[0], 2);

        assert_eq!(vector.remove(0), 2);
        assert_eq!(vector.size(), 0);
    }

    #[test]
    #[should_panic]
    fn remove_element_by_index_invalid() {
        let mut vector = MiniVec::<i32>::new();

        vector.remove(0);
    }

    #[test]
    fn correct_realloc_call() {
        let mut vector = MiniVec::<i32>::new();

        for i in 0..1000 {
            vector.push(i);
        }
    }

    #[test]
    fn mutable_indexing_valid() {
        let mut vector = MiniVec::<i32>::new();

        vector.push(0);
        vector.push(1);
        vector.push(2);
        vector.push(3);

        vector[0] = 200;
        assert_eq!(vector[0], 200);
        assert_eq!(vector.size(), 4);
        assert_eq!(vector.capacity(), 4);

        vector[3] = 120;
        assert_eq!(vector[3], 120);
        assert_eq!(vector.size(), 4);
        assert_eq!(vector.capacity(), 4);

        vector[2] = 90;
        assert_eq!(vector[2], 90);
        assert_eq!(vector.size(), 4);
        assert_eq!(vector.capacity(), 4);
    }

    #[test]
    #[should_panic]
    fn mutable_indexing_invalid() {
        let mut vector = MiniVec::<i32>::new();

        vector[1000] = 30;
    }
}
