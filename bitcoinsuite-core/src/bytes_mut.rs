use std::hash::Hash;

use crate::{ByteArray, Bytes};

#[derive(Debug, Clone, Default)]
pub struct BytesMut {
    data: bytes::BytesMut,
}

impl PartialEq for BytesMut {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl Eq for BytesMut {}

impl Hash for BytesMut {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl BytesMut {
    pub fn new() -> Self {
        BytesMut {
            data: bytes::BytesMut::new(),
        }
    }

    pub fn from_bytes_mut(data: impl Into<bytes::BytesMut>) -> Self {
        BytesMut { data: data.into() }
    }

    pub fn put_bytes_mut(&mut self, data: impl Into<BytesMut>) {
        self.data.unsplit(data.into().data);
    }

    pub fn put_bytes(&mut self, data: Bytes) {
        self.data.extend_from_slice(&data);
    }

    pub fn put_byte_array<const N: usize>(&mut self, byte_array: ByteArray<N>) {
        self.data.extend_from_slice(&byte_array);
    }

    pub fn put_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    pub fn freeze(self) -> Bytes {
        Bytes::from_bytes(self.data.freeze())
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}
