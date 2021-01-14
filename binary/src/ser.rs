use crate::attr::Attrs;
use crate::{BufMut, Result};

pub trait BinSerialize {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()>;
}

pub fn encode_to_bytes<T>(t: T) -> Result<Vec<u8>>
where
    T: BinSerialize,
{
    let mut buf = vec![];
    t.encode_to(&mut buf, Attrs::zero())?;
    Ok(buf)
}
