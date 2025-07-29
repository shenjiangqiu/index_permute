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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(feature = "parallel")]
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
/// You can also create a `PermuteIndex` from a slice:
/// ```
/// use index_permute::PermuteIndex;
/// let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
/// let vec_data = vec![0, 1, 2];
/// let _ = PermuteIndex::try_new(&vec_data);
/// let _ = PermuteIndex::try_new(&[0, 1, 2][..]);
/// ```
#[derive(Debug, Clone)]
pub struct PermuteIndex<'a> {
    data: &'a [usize],
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
impl<'a> PermuteIndex<'a> {
    fn check_index(index: &[usize]) -> bool {
        // make sure all indices are unique and from 0 to len-1
        let len = index.len();
        let mut seen = vec![false; len];
        for &i in index {
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
    /// You can also create a `PermuteIndex` from a slice:
    /// ```
    /// use index_permute::PermuteIndex;
    /// let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
    /// let _ = PermuteIndex::try_new(&[0, 1, 2][..]);
    /// ```
    pub fn try_new(index: &'a [usize]) -> Result<Self, PermuteError> {
        if Self::check_index(index) {
            Ok(PermuteIndex { data: index })
        } else {
            Err(PermuteError::InvalidIndex)
        }
    }

    /// Creates a new [`PermuteIndex`] without checking the validity of the index.
    /// also see [`PermuteIndex::try_new`].
    pub unsafe fn new_unchecked(index: &'a [usize]) -> Self {
        // This function is unsafe because it does not check the validity of the index.
        // It should only be used when you are sure that the index is valid.
        PermuteIndex { data: index }
    }

    fn generate_swaps(&self) -> Vec<(usize, usize)> {
        let mut visited = vec![false; self.data.len()];
        let mut swaps = vec![];

        for i in 0..self.data.len() {
            if visited[i] || self.data[i] == i {
                continue;
            }

            let mut x = i;

            while !visited[self.data[x]] {
                visited[x] = true;
                x = self.data[x];
                swaps.push((i, x));
            }
        }

        swaps.reverse();
        swaps
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
pub fn try_order_by_index_inplace<T>(
    data: &mut [T],
    index: PermuteIndex,
) -> Result<(), PermuteError> {
    if index.data.len() != data.len() {
        return Err(PermuteError::LengthMismatch);
    }

    for (a, b) in index.generate_swaps() {
        data.swap(a, b);
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
pub fn order_by_index_inplace<T>(data: &mut [T], index: PermuteIndex)
where
    T: Send,
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

pub fn try_order_by_index_parallel_inplace_with_threads<T>(
    data: &mut [T],
    index: PermuteIndex,
    num_threads: usize,
) -> Result<(), PermuteError>
where
    T: Sync + Send,
{
    let len = data.len();

    if len < 10_000 || num_threads <= 1 {
        return try_order_by_index_inplace(data, index);
    }

    if len != index.data.len() {
        return Err(PermuteError::LengthMismatch);
    }

    let indices = index.data;
    let chunk_size = (len + num_threads - 1) / num_threads;

    // Create buffer with proper initialization
    let mut raw_buffer: Vec<T> = Vec::with_capacity(len);
    let buffer = raw_buffer.spare_capacity_mut();

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
        // SAFETY: We are copying from a valid memory location to another valid memory location
        ptr::copy_nonoverlapping(buffer.as_ptr() as *const T, data.as_mut_ptr(), len);
    }



    Ok(())
}
/// Same as [`try_order_by_index_parallel_inplace_with_threads`] but uses the number of available CPU cores.
/// Only valid when features `parallel` is enabled.
pub fn try_order_by_index_parallel_inplace<T>(
    data: &mut [T],
    index: PermuteIndex,
) -> Result<(), PermuteError>
where
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
    fn test_generate_swaps() {
        let index = PermuteIndex::try_new(&[2, 0, 1, 4, 3]).unwrap();
        assert_eq!(index.generate_swaps(), vec![(3, 4), (0, 1), (0, 2)]);
    }

    #[test]
    fn test_permute_index() {
        let _ = PermuteIndex::try_new(&[0usize, 2, 1]);
        let vec_data = vec![0, 1, 2];
        let _ = PermuteIndex::try_new(&vec_data);
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
        let index_vec = (0..1000).rev().collect::<Vec<_>>();
        let index = PermuteIndex::try_new(&index_vec).unwrap();
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
        let index_vec = (0..test_size).rev().collect::<Vec<_>>();
        let index = PermuteIndex::try_new(&index_vec).unwrap();

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
