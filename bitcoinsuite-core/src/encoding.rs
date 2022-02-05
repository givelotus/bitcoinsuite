use std::convert::TryInto;

use crate::{Bytes, BytesError, BytesMut};

pub fn read_compact_size(bytes: &mut Bytes) -> Result<u64, BytesError> {
    let first_byte = bytes.split_to(1)?[0];
    match first_byte {
        0..=0xfc => Ok(first_byte as u64),
        0xfd => Ok(u16::from_le_bytes(bytes.split_to_array::<2>()?.array()) as u64),
        0xfe => Ok(u32::from_le_bytes(bytes.split_to_array::<4>()?.array()) as u64),
        0xff => Ok(u64::from_le_bytes(bytes.split_to_array::<8>()?.array())),
    }
}

pub fn read_compact_size_slice(slice: &[u8]) -> Option<(usize, u64)> {
    let first_byte = *slice.get(0)?;
    match first_byte {
        0..=0xfc => Some((1, first_byte as u64)),
        0xfd => Some((
            2,
            u16::from_le_bytes(slice.get(1..3)?.try_into().unwrap()) as u64,
        )),
        0xfe => Some((
            4,
            u32::from_le_bytes(slice.get(1..5)?.try_into().unwrap()) as u64,
        )),
        0xff => Some((
            8,
            u64::from_le_bytes(slice.get(1..9)?.try_into().unwrap()) as u64,
        )),
    }
}

pub fn write_compact_size(bytes: &mut BytesMut, size: u64) {
    match size {
        0..=0xfc => bytes.put_slice(&[size as u8]),
        0xfd..=0xffff => {
            bytes.put_slice(&[0xfd]);
            bytes.put_slice(&(size as u16).to_le_bytes());
        }
        0x10000..=0xffff_ffff => {
            bytes.put_slice(&[0xfe]);
            bytes.put_slice(&(size as u32).to_le_bytes());
        }
        _ => {
            bytes.put_slice(&[0xff]);
            bytes.put_slice(&size.to_le_bytes());
        }
    }
}
