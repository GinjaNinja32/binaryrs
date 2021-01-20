use crate::attr::Attrs;
use crate::{BinWrite, Result};

pub trait BinSerialize {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()>;
}

pub fn encode_to_bytes<T>(t: T) -> Result<Vec<u8>>
where
    T: BinSerialize,
{
    let mut buf = vec![];
    t.encode_to(&mut buf, Attrs::zero())?;
    Ok(buf)
}

pub fn encode_to_stream<T>(t: T, s: &mut dyn BinWrite) -> Result<()>
where
    T: BinSerialize,
{
    t.encode_to(s, Attrs::zero())
}
