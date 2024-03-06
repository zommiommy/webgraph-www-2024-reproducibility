/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

/// A circular buffer which is used to keep the backreferences both in
/// sequential reads and for compressing during writes.
/// For efficency reasons, we re-use the allocated buffers to avoid pressure
/// over the allocator.
#[derive(Clone)]
pub(crate) struct CircularBufferVec {
    data: Vec<Vec<usize>>,
}

impl CircularBufferVec {
    /// Create a new circular buffer that can hold `len` values. This should be
    /// equal to the compression windows + 1 so there is space for the new data.
    pub(crate) fn new(len: usize) -> Self {
        Self {
            data: (0..len)
                .map(|_| Vec::with_capacity(100))
                .collect::<Vec<_>>(),
        }
    }

    /// Take the buffer to write the neighbours of the new node
    pub(crate) fn take(&mut self, index: usize) -> Vec<usize> {
        let idx = index % self.data.len();
        let mut res = core::mem::take(&mut self.data[idx]);
        res.clear();
        res
    }

    /// Put it back in the buffer so it can be read
    pub(crate) fn push(&mut self, index: usize, data: Vec<usize>) -> &[usize] {
        let idx = index % self.data.len();
        self.data[idx] = data;
        &self.data[idx]
    }
}

impl core::ops::Index<usize> for CircularBufferVec {
    type Output = [usize];

    #[inline]
    fn index(&self, node_id: usize) -> &Self::Output {
        let idx = node_id % self.data.len();
        &self.data[idx]
    }
}

impl core::ops::Index<isize> for CircularBufferVec {
    type Output = [usize];

    #[inline]
    fn index(&self, node_id: isize) -> &Self::Output {
        // TODO!: add checks
        let idx = node_id.rem_euclid(self.data.len() as isize) as usize;
        &self.data[idx]
    }
}

/// A circular buffer which is used to keep the backreferences both in
/// sequential reads and for compressing during writes.
/// For efficency reasons, we re-use the allocated buffers to avoid pressure
/// over the allocator.
pub(crate) struct CircularBuffer<T: Default> {
    data: Vec<T>,
}

impl<T: Default> CircularBuffer<T> {
    /// Create a new circular buffer that can hold `len` values. This should be
    /// equal to the compression windows + 1 so there is space for the new data.
    pub(crate) fn new(len: usize) -> Self {
        Self {
            data: (0..len).map(|_| T::default()).collect::<Vec<_>>(),
        }
    }
}

impl<T: Default> core::ops::Index<usize> for CircularBuffer<T> {
    type Output = T;

    #[inline]
    fn index(&self, node_id: usize) -> &Self::Output {
        let idx = node_id % self.data.len();
        &self.data[idx]
    }
}

impl<T: Default> core::ops::IndexMut<usize> for CircularBuffer<T> {
    #[inline]
    fn index_mut(&mut self, node_id: usize) -> &mut Self::Output {
        let idx = node_id % self.data.len();
        &mut self.data[idx]
    }
}

impl<T: Default> core::ops::Index<isize> for CircularBuffer<T> {
    type Output = T;

    #[inline]
    fn index(&self, node_id: isize) -> &Self::Output {
        let idx = node_id.rem_euclid(self.data.len() as isize) as usize;
        &self.data[idx]
    }
}

impl<T: Default> core::ops::IndexMut<isize> for CircularBuffer<T> {
    #[inline]
    fn index_mut(&mut self, node_id: isize) -> &mut Self::Output {
        let idx = node_id.rem_euclid(self.data.len() as isize) as usize;
        &mut self.data[idx]
    }
}
