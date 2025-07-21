//! - A tool to reorder an array by an index array inplace when the elements are not [`Clone`] or [`Copy`].
//! - lock-free parallel implementation.
//! ## Example
//! ```
//! use index_permute::PermuteIndex;
//! let index = PermuteIndex::try_new(&[2, 0, 1
//! ]).unwrap();
//! let mut data = vec![10, 20, 30];
//! index_permute::order_by_index_inplace(&mut data, index);
//! assert_eq!(data, vec![30, 10, 20]);
//! ```
#![deny(missing_docs)]
#![deny(warnings)]

use std::ptr;
use thiserror::Error;
mod macro_rules;


/// A struct to hold a permutation index.
/// The index must be unique and in the range of `0..len`, where `len` is the length of the data to be permuted.
/// This struct is used to ensure that the index is valid before performing any operations on the data.
/// It can be created using [`PermuteIndex::try_new`], which checks the validity of the index.
/// If the index is invalid, it returns a [`PermuteError::InvalidIndex`] error.
/// The index can be used to permute data using the [`try_order_by_index_inplace`] function.
/// The index length must match the data length, otherwise it returns a [`PermuteError::LengthMismatch`] error.
/// The [`order_by_index_inplace`] function is a convenience function that panics if the index is invalid or the lengths do not match.
/// # Example
/// ```
/// use index_permute::PermuteIndex;
/// let index = PermuteIndex::try_new(&[2, 0, 1
/// ]).unwrap();
/// let mut data = vec![10, 20, 30];
/// index_permute::order_by_index_inplace(&mut data, index);
/// assert_eq!(data, vec![30, 10, 20]);
/// ```
///
/// You can also create a `PermuteIndex` from a vector or slice:
/// ```
/// use index_permute::PermuteIndex;
/// let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
/// let _ = PermuteIndex::try_new(vec![0, 1, 2]);
/// let _ = PermuteIndex::try_new(&vec![0, 1, 2]);
/// let _ = PermuteIndex::try_new(&[0, 1, 2][..]);
/// ```
#[derive(Debug, Clone)]
pub struct PermuteIndex<T> {
    data: T,
}

/// An error type for [`PermuteIndex`] and [`try_order_by_index_inplace`].
#[derive(Debug, Error)]
pub enum PermuteError {
    /// An error indicating that the index is invalid.
    #[error("Invalid index: indices must be unique and in 0..len")]
    InvalidIndex,
    /// An error indicating that the index length does not match the data length.
    #[error("Index length must match data length")]
    LengthMismatch,
}
impl<T> PermuteIndex<T>
where
    T: AsRef<[usize]>,
{
    fn check_index(index: &T) -> bool {
        // make sure all indices are unique and from 0 to len-1
        let indices = index.as_ref();
        let len = indices.len();
        let mut seen = vec![false; len];
        for &i in indices {
            if i >= len || seen[i] {
                return false; // index out of bounds or duplicate
            }
            seen[i] = true;
        }
        true
    }
    /// Creates a new [`PermuteIndex`] if the index is valid.
    /// Returns [`PermuteError::InvalidIndex`] if the index is not valid.
    /// The index must be unique and in the range of `0..len`, where `len` is the length of the data to be permuted.
    /// The index can be used to permute data using the [`try_order_by_index_inplace`] function.
    /// The index length must match the data length, otherwise it returns [`PermuteError::LengthMismatch`] error.
    /// The [`order_by_index_inplace`] function is a convenience function that panics if the index is invalid or the lengths do not match.
    /// # Example
    /// ```
    /// use index_permute::PermuteIndex;
    /// let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();
    /// let mut data = vec![10, 20, 30];
    /// index_permute::order_by_index_inplace(&mut data, index);
    /// assert_eq!(data, vec![30, 10, 20]);
    /// ```
    /// You can also create a `PermuteIndex` from a vector or slice:
    /// ```
    /// use index_permute::PermuteIndex;
    /// let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
    /// let _ = PermuteIndex::try_new(vec![0, 1, 2]);
    /// let _ = PermuteIndex::try_new(&vec![0, 1, 2]);
    /// let _ = PermuteIndex::try_new(&[0, 1, 2][..]);
    /// ```
    pub fn try_new(index: T) -> Result<Self, PermuteError> {
        if Self::check_index(&index) {
            Ok(PermuteIndex { data: index })
        } else {
            Err(PermuteError::InvalidIndex)
        }
    }

    /// Creates a new [`PermuteIndex`] without checking the validity of the index.
    /// also see [`PermuteIndex::try_new`].
    pub unsafe fn new_unchecked(index: T) -> Self {
        // This function is unsafe because it does not check the validity of the index.
        // It should only be used when you are sure that the index is valid.
        PermuteIndex { data: index }
    }
}

/// Reorders the data in place according to the given index.
/// First create a [`PermuteIndex`], then, it reorders the data in place
/// # Example
/// ```
/// use index_permute::PermuteIndex;
/// let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();
/// let mut data = vec![10, 20, 30];
/// index_permute::try_order_by_index_inplace(&mut data, index).unwrap();
/// assert_eq!(data, vec![30, 10, 20]);
/// ```
pub fn try_order_by_index_inplace<T, I>(
    data: &mut [T],
    index: PermuteIndex<I>,
) -> Result<(), PermuteError>
where
    I: AsRef<[usize]>,
{
    let indices = index.data.as_ref();
    let len = data.len();
    if indices.len() != len {
        return Err(PermuteError::LengthMismatch);
    }

    // SAFETY: indices are unique and a valid permutation of 0..len,
    // so we can move elements without overlap.

    // Create a Vec<T> with uninitialized memory
    let mut temp: Vec<T> = Vec::with_capacity(len);
    unsafe {
        temp.set_len(len);

        for (i, &idx) in indices.iter().enumerate() {
            // Move from data[idx] to temp[i]
            ptr::write(
                temp.get_unchecked_mut(i),
                ptr::read(data.get_unchecked(idx)),
            );
        }

        // Move back from temp to data
        for i in 0..len {
            ptr::write(data.get_unchecked_mut(i), ptr::read(temp.get_unchecked(i)));
        }
        // should not forget `temp`, it should be dropped, but the items should not be deallocated, because they are moved to data
        // so we prevent deallocation of `temp` by setting its length to 0
        temp.set_len(0); // Prevent deallocation of temp
    }
    Ok(())
}

/// A convenience function that panics if the index is invalid or the lengths do not match.
/// It is recommended to use [`try_order_by_index_inplace`] for error handling.
/// # Example
/// ```
/// use index_permute::PermuteIndex;
/// let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();
/// let mut data = vec![10, 20, 30];
/// index_permute::order_by_index_inplace(&mut data, index);
/// assert_eq!(data, vec![30, 10, 20]);
/// ```
pub fn order_by_index_inplace<T, I>(data: &mut [T], index: PermuteIndex<I>)
where
    I: AsRef<[usize]>,
{
    if let Err(e) = try_order_by_index_inplace(data, index) {
        panic!("Failed to order by index: {}", e);
    }
}

cfg_parallel! {
/// Only valid when features `parallel` is enabled.
/// A parallel version of [`try_order_by_index_inplace`].
/// # Parameters
/// - `data`: The data to be permuted.
/// - `index`: The permutation index, which must be a valid [`PermuteIndex`].
/// - `num_threads`: The number of threads to use for parallel processing.
/// # Returns
/// - `Ok(())` if the operation was successful.
/// - `Err(PermuteError)` if the index is invalid or the lengths do not match.
///
/// Improved parallel version

pub fn try_order_by_index_parallel_inplace_with_threads<T, I>(
    data: &mut [T],
    index: PermuteIndex<I>,
    num_threads: usize,
) -> Result<(), PermuteError>
where
    I: AsRef<[usize]>,
    T: Sync + Send,
{
    let len = data.len();

    if len < 10_000 || num_threads <= 1 {
        return try_order_by_index_inplace(data, index);
    }

    if len != index.data.as_ref().len() {
        return Err(PermuteError::LengthMismatch);
    }

    let indices = index.data.as_ref();
    let chunk_size = (len + num_threads - 1) / num_threads;

    // Create buffer with proper initialization
    let mut buffer: Vec<std::mem::MaybeUninit<T>> = Vec::with_capacity(len);
    buffer.resize_with(len, std::mem::MaybeUninit::uninit);

    // Convert data to shared reference for reading
    let data_ref = &*data;

    let index_chunks = indices.chunks(chunk_size);
    let buffer_chunks = buffer.chunks_mut(chunk_size);
    // Parallel phase 1: Read from data according to indices, write to buffer
    std::thread::scope(|s| {
        for (indices_slice, buffer_slice) in index_chunks.zip(buffer_chunks) {
            s.spawn(move || {
                for (i, &src_idx) in indices_slice.iter().enumerate() {
                    // Safe read from source
                    let value = unsafe { ptr::read(data_ref.get_unchecked(src_idx)) };
                    // Safe write to buffer
                    unsafe {
                        buffer_slice.get_unchecked_mut(i).write(value);
                    }
                }
            });
        }
    });

    // Now buffer contains all the reordered data
    // Phase 2: Move from buffer back to data
    unsafe {
        for i in 0..len {
            let value = buffer.get_unchecked(i).as_ptr().read();
            ptr::write(data.get_unchecked_mut(i), value);
        }
    }

    // Buffer will be dropped but MaybeUninit won't drop the T values
    std::mem::drop(buffer);

    Ok(())
}
/// Same as [`try_order_by_index_parallel_inplace_with_threads`] but uses the number of available CPU cores.
/// Only valid when features `parallel` is enabled.
pub fn try_order_by_index_parallel_inplace<T, I>(
    data: &mut [T],
    index: PermuteIndex<I>,
) -> Result<(), PermuteError>
where
    I: AsRef<[usize]>,
    T: Sync + Send,
{
    let num_threads = num_cpus::get();
    try_order_by_index_parallel_inplace_with_threads(data, index, num_threads)
}
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permute_index() {
        let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
        let _ = PermuteIndex::try_new(vec![0, 1, 2]);
        let _ = PermuteIndex::try_new(&vec![0, 1, 2]);
        let _ = PermuteIndex::try_new(&[0, 1, 2][..]);
    }

    #[test]
    fn test_permute_order() {
        let mut data = vec![10, 20, 30];
        let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();
        assert!(try_order_by_index_inplace(&mut data, index).is_ok());
        assert_eq!(data, vec![30, 10, 20]);
    }

    #[test]
    fn test_drop() {
        struct DropTest {
            value: usize,
        }
        impl Drop for DropTest {
            fn drop(&mut self) {
                println!("Dropping {}", self.value);
            }
        }
        let mut data = vec![
            DropTest { value: 1 },
            DropTest { value: 2 },
            DropTest { value: 3 },
        ];
        let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();

        // now, there should be no drop
        assert!(try_order_by_index_inplace(&mut data, index).is_ok());
        println!("no drop should happen here");

        assert_eq!(data[0].value, 3);
        assert_eq!(data[1].value, 1);
        assert_eq!(data[2].value, 2);
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_order_by_index_parallel() {
        let mut data = (0..1000).collect::<Vec<_>>();
        let index = PermuteIndex::try_new((0..1000).rev().collect::<Vec<_>>()).unwrap();
        assert!(try_order_by_index_parallel_inplace(&mut data, index).is_ok());
        assert_eq!(data, (0..1000).rev().collect::<Vec<_>>());
    }

    #[test]
    #[cfg(feature = "parallel")]
    fn test_order_by_index_drop() {
        struct DropTest {
            value: usize,
        }
        impl Drop for DropTest {
            fn drop(&mut self) {
                print!(".",);
            }
        }
        let test_size = 10001;
        let mut data = (0..test_size)
            .map(|i| DropTest { value: i })
            .collect::<Vec<_>>();
        let index = PermuteIndex::try_new((0..test_size).rev().collect::<Vec<_>>()).unwrap();

        // now, there should be no drop
        try_order_by_index_parallel_inplace_with_threads(&mut data, index, 4).unwrap();
        println!("no drop should happen here");

        // assert_eq!(data[0].value, 999);
        // assert_eq!(data[1].value, 998);
        // assert_eq!(data[2].value, 997);
        for i in 0..test_size {
            assert_eq!(data[i].value, test_size - 1 - i);
        }
    }
}
