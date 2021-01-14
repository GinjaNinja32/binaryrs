use crate::attr::Attrs;
use crate::{Buf, Result};

pub trait BinDeserialize: Sized {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self>;
}

pub fn decode_from_bytes<T>(mut buf: &[u8]) -> Result<T>
where
    T: BinDeserialize,
{
    T::decode_from(&mut buf, Attrs::zero())
}