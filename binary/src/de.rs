use crate::attr::Attrs;
use crate::{BinRead, Result};

pub trait BinDeserialize: Sized {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self>;
}

pub fn decode_from_bytes<T>(mut buf: &[u8]) -> Result<T>
where
    T: BinDeserialize,
{
    T::decode_from(&mut buf, Attrs::zero())
}

pub fn decode_from_stream<T>(s: &mut dyn BinRead) -> Result<T>
where
    T: BinDeserialize,
{
    T::decode_from(s, Attrs::zero())
}
