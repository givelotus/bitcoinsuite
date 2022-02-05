use std::{hash::Hash, ops::Deref};

use thiserror::Error;

use crate::ByteArray;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BytesError {
    #[error("Index {split_idx} is out of bounds for array with length {len}")]
    InvalidSplit { split_idx: usize, len: usize },
}

#[derive(Debug, Clone, Default)]
pub struct Bytes {
    data: bytes::Bytes,
}

impl Deref for Bytes {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl Eq for Bytes {}

impl Hash for Bytes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl Bytes {
    pub fn new() -> Self {
        Bytes {
            data: bytes::Bytes::new(),
        }
    }

    pub fn from_bytes(data: impl Into<bytes::Bytes>) -> Self {
        Bytes { data: data.into() }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Bytes {
            data: data.to_vec().into(),
        }
    }

    pub fn split_to(&mut self, at: usize) -> Result<Bytes, BytesError> {
        if self.data.len() < at {
            return Err(BytesError::InvalidSplit {
                split_idx: at,
                len: self.data.len(),
            });
        }
        Ok(Bytes::from_bytes(self.data.split_to(at)))
    }

    pub fn split_to_array<const N: usize>(&mut self) -> Result<ByteArray<N>, BytesError> {
        if self.data.len() < N {
            return Err(BytesError::InvalidSplit {
                split_idx: N,
                len: self.data.len(),
            });
        }
        Ok(ByteArray::new(
            self.data.split_to(N).as_ref().try_into().unwrap(),
        ))
    }

    pub fn hex(&self) -> String {
        hex::encode(&self.data)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Bytes::from_bytes(vec)
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(arr: [u8; N]) -> Self {
        Bytes::from_slice(&arr)
    }
}

impl<'a> From<&'a [u8]> for Bytes {
    fn from(slice: &'a [u8]) -> Self {
        Bytes::from_bytes(slice.to_vec())
    }
}
