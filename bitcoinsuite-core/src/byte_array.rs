use std::{convert::TryInto, hash::Hash, ops::Deref};

use crate::{BitcoinSuiteError, Result};

#[derive(Debug, Clone)]
pub struct ByteArray<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> ByteArray<N> {
    pub fn new(data: [u8; N]) -> Self {
        ByteArray { data }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        let array: [u8; N] = slice
            .try_into()
            .map_err(|_| BitcoinSuiteError::InvalidSize {
                expected: N,
                actual: slice.len(),
            })?;
        Ok(ByteArray::new(array))
    }

    pub fn array(&self) -> [u8; N] {
        self.data
    }

    pub fn as_array(&self) -> &[u8; N] {
        &self.data
    }

    pub fn hex(&self) -> String {
        hex::encode(&self.data)
    }
}

impl<const N: usize> Default for ByteArray<N> {
    fn default() -> Self {
        ByteArray { data: [0; N] }
    }
}

impl<const N: usize> Deref for ByteArray<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize> PartialEq for ByteArray<N> {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl<const N: usize> Eq for ByteArray<N> {}

impl<const N: usize> Hash for ByteArray<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl<const N: usize> From<[u8; N]> for ByteArray<N> {
    fn from(arr: [u8; N]) -> Self {
        ByteArray::new(arr)
    }
}
