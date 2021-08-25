use crate::{
    encoding::{read_compact_size, write_compact_size},
    ByteArray, Bytes, BytesMut, Result,
};

pub trait BitcoinCode: Sized {
    fn ser_to(&self, bytes: &mut BytesMut);
    fn deser(data: &mut Bytes) -> Result<Self>;
    fn ser(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        self.ser_to(&mut bytes);
        bytes.freeze()
    }
}

impl BitcoinCode for Bytes {
    fn ser_to(&self, bytes: &mut BytesMut) {
        write_compact_size(bytes, self.len() as u64);
        bytes.put_slice(self.as_ref());
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        let size = read_compact_size(data)?;
        Ok(data.split_to(size as usize)?)
    }
}

impl<const N: usize> BitcoinCode for ByteArray<N> {
    fn ser_to(&self, bytes: &mut BytesMut) {
        bytes.put_slice(self.as_ref());
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(data.split_to_array::<N>()?)
    }
}

impl<T: BitcoinCode> BitcoinCode for Vec<T> {
    fn ser_to(&self, bytes: &mut BytesMut) {
        write_compact_size(bytes, self.len() as u64);
        for part in self {
            part.ser_to(bytes);
        }
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        let size = read_compact_size(data)? as usize;
        let mut vec = Vec::with_capacity(size);
        for _ in 0..size {
            let item = T::deser(data)?;
            vec.push(item);
        }
        Ok(vec)
    }
}

impl BitcoinCode for bool {
    fn ser_to(&self, bytes: &mut BytesMut) {
        bytes.put_slice(&[*self as u8]);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(data.split_to(1)?[0] != 0)
    }
}

macro_rules! integer_impls {
    ($($T:ident $SIZE:literal,)+) => {
        $(
            impl BitcoinCode for $T {
                fn ser_to(&self, bytes: &mut BytesMut) {
                    bytes.put_slice(&self.to_le_bytes())
                }

                fn deser(data: &mut Bytes) -> Result<Self> {
                    let array = data.split_to_array::<$SIZE>()?;
                    let value = $T::from_le_bytes(array.array());
                    Ok(value)
                }
            }
        )+
    }
}

integer_impls! {
    u8 1, i8 1, u16 2, i16 2, u32 4, i32 4, u64 8, i64 8, u128 16, i128 16,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::{BitcoinCode, ByteArray, Bytes};

    fn verify_ser<T: BitcoinCode + PartialEq + Debug>(a: T, b: &[u8]) {
        assert_eq!(a.ser().as_ref(), b);
        let deser = T::deser(&mut b.into()).expect("Deser fail");
        assert_eq!(a, deser);
    }

    #[test]
    fn test_ser_bytes() {
        verify_ser(Bytes::new(), &[0]);
        verify_ser(Bytes::from_slice(&[1]), &[1, 1]);
        verify_ser(Bytes::from_slice(&[1, 2, 3]), &[3, 1, 2, 3]);
        verify_ser(
            Bytes::from_slice(&[4; 0xfc]),
            &[[0xfc].as_ref(), &[4; 0xfc]].concat(),
        );
        verify_ser(
            Bytes::from_slice(&[5; 0xfd]),
            &[[0xfd, 0xfd, 0].as_ref(), &[5; 0xfd]].concat(),
        );
        verify_ser(
            Bytes::from_slice(&[6; 0xfe]),
            &[[0xfd, 0xfe, 0].as_ref(), &[6; 0xfe]].concat(),
        );
        verify_ser(
            Bytes::from_slice(&vec![7; 0xffff]),
            &[[0xfd, 0xff, 0xff].as_ref(), &vec![7; 0xffff]].concat(),
        );
        verify_ser(
            Bytes::from_slice(&vec![8; 0x10000]),
            &[[0xfe, 0, 0, 1, 0].as_ref(), &vec![8; 0x10000]].concat(),
        );
    }

    #[test]
    fn test_ser_byte_array() {
        verify_ser(ByteArray::new([]), &[]);
        verify_ser(ByteArray::new([1]), &[1]);
        verify_ser(ByteArray::new([1, 2]), &[1, 2]);
        verify_ser(ByteArray::new([1, 2, 3]), &[1, 2, 3]);
        verify_ser(ByteArray::new([4; 32]), &[4; 32]);
        verify_ser(ByteArray::new([5; 0xff]), &[5; 0xff]);
    }

    #[test]
    fn test_ser_vec() {
        verify_ser(Vec::<u32>::new(), &[0]);
        verify_ser(vec![1u8], &[1, 1]);
        verify_ser(vec![1u32], &[1, 1, 0, 0, 0]);
        verify_ser(vec![1u8, 2, 3], &[3, 1, 2, 3]);
        verify_ser(vec![1u16, 2, 3], &[3, 1, 0, 2, 0, 3, 0]);
        let vec_bytes = vec![
            Bytes::new(),
            Bytes::from_slice(&[1]),
            Bytes::from_slice(&[1, 2, 3]),
        ];
        verify_ser(vec_bytes, &[3, 0, 1, 1, 3, 1, 2, 3]);
    }

    #[test]
    fn test_ser_bool() {
        verify_ser(true, &[1]);
        verify_ser(false, &[0]);
        for i in 2u8..=255 {
            let b = bool::deser(&mut [i].into()).expect("Deser failed");
            assert!(b);
        }
    }

    #[test]
    fn test_ser_integers() {
        verify_ser(128u8, &[128]);
        verify_ser(123u8, &[123]);
        verify_ser(123i8, &[123]);
        verify_ser(-123i8, &[133]);
        verify_ser(0x1234u16, &[0x34, 0x12]);
        verify_ser(0x9234u16, &[0x34, 0x92]);
        verify_ser(0x1234i16, &[0x34, 0x12]);
        verify_ser(-0x1234i16, &[0xcc, 0xed]);
        verify_ser(0x12345678u32, &[0x78, 0x56, 0x34, 0x12]);
        verify_ser(0x92345678u32, &[0x78, 0x56, 0x34, 0x92]);
        verify_ser(0x12345678i32, &[0x78, 0x56, 0x34, 0x12]);
        verify_ser(-0x12345678i32, &[0x88, 0xa9, 0xcb, 0xed]);
        verify_ser(
            0x1234567890abcdefu64,
            &[0xef, 0xcd, 0xab, 0x90, 0x78, 0x56, 0x34, 0x12],
        );
        verify_ser(
            0x9234567890abcdefu64,
            &[0xef, 0xcd, 0xab, 0x90, 0x78, 0x56, 0x34, 0x92],
        );
        verify_ser(
            0x1234567890abcdefi64,
            &[0xef, 0xcd, 0xab, 0x90, 0x78, 0x56, 0x34, 0x12],
        );
        verify_ser(
            -0x1234567890abcdefi64,
            &[0x11, 0x32, 0x54, 0x6f, 0x87, 0xa9, 0xcb, 0xed],
        );
        verify_ser(
            0x1234567890abcdeffedcba0123456789u128,
            &[
                0x89, 0x67, 0x45, 0x23, 0x01, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x90, 0x78, 0x56,
                0x34, 0x12,
            ],
        );
        verify_ser(
            0x9234567890abcdeffedcba0123456789u128,
            &[
                0x89, 0x67, 0x45, 0x23, 0x01, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x90, 0x78, 0x56,
                0x34, 0x92,
            ],
        );
        verify_ser(
            0x1234567890abcdeffedcba0123456789i128,
            &[
                0x89, 0x67, 0x45, 0x23, 0x01, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x90, 0x78, 0x56,
                0x34, 0x12,
            ],
        );
        verify_ser(
            -0x1234567890abcdeffedcba0123456789i128,
            &[
                0x77, 0x98, 0xba, 0xdc, 0xfe, 0x45, 0x23, 0x01, 0x10, 0x32, 0x54, 0x6f, 0x87, 0xa9,
                0xcb, 0xed,
            ],
        );
    }
}
