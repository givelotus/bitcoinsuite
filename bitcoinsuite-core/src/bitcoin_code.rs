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

macro_rules! array_impls {
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

array_impls! {
    u8 1, i8 1, u16 2, i16 2, u32 4, i32 4, u64 8, i64 8, u128 16, i128 16,
}
