use crate::attr::{Attrs, Endian};
use crate::{BinDeserialize, BinError, BinSerialize, Buf, BufMut, Result};

impl BinSerialize for bool {
    fn encode_to(&self, buf: &mut dyn BufMut, _attrs: Attrs) -> Result<()> {
        buf.put_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}
impl BinDeserialize for bool {
    fn decode_from(buf: &mut dyn Buf, _attrs: Attrs) -> Result<Self> {
        buf.req(1)?;
        Ok(buf.get_u8() != 0)
    }
}

impl BinSerialize for i8 {
    fn encode_to(&self, buf: &mut dyn BufMut, _attrs: Attrs) -> Result<()> {
        buf.put_i8(*self);
        Ok(())
    }
}
impl BinDeserialize for i8 {
    fn decode_from(buf: &mut dyn Buf, _attrs: Attrs) -> Result<Self> {
        buf.req(1)?;
        Ok(buf.get_i8())
    }
}

impl BinSerialize for u8 {
    fn encode_to(&self, buf: &mut dyn BufMut, _attrs: Attrs) -> Result<()> {
        buf.put_u8(*self);
        Ok(())
    }
}
impl BinDeserialize for u8 {
    fn decode_from(buf: &mut dyn Buf, _attrs: Attrs) -> Result<Self> {
        buf.req(1)?;
        Ok(buf.get_u8())
    }
}

impl BinSerialize for u16 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u16(*self),
            Endian::Little => buf.put_u16_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for u16 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(2)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_u16(),
            Endian::Little => buf.get_u16_le(),
        })
    }
}

impl BinSerialize for i16 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i16(*self),
            Endian::Little => buf.put_i16_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for i16 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(2)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_i16(),
            Endian::Little => buf.get_i16_le(),
        })
    }
}

impl BinSerialize for u32 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u32(*self),
            Endian::Little => buf.put_u32_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for u32 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(4)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_u32(),
            Endian::Little => buf.get_u32_le(),
        })
    }
}

impl BinSerialize for i32 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i32(*self),
            Endian::Little => buf.put_i32_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for i32 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(4)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_i32(),
            Endian::Little => buf.get_i32_le(),
        })
    }
}

impl BinSerialize for u64 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u64(*self),
            Endian::Little => buf.put_u64_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for u64 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(8)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_u64(),
            Endian::Little => buf.get_u64_le(),
        })
    }
}

impl BinSerialize for i64 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i64(*self),
            Endian::Little => buf.put_i64_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for i64 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(8)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_i64(),
            Endian::Little => buf.get_i64_le(),
        })
    }
}

impl BinSerialize for f32 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_f32(*self),
            Endian::Little => buf.put_f32_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for f32 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(4)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_f32(),
            Endian::Little => buf.get_f32_le(),
        })
    }
}

impl BinSerialize for f64 {
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_f64(*self),
            Endian::Little => buf.put_f64_le(*self),
        }
        Ok(())
    }
}
impl BinDeserialize for f64 {
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        buf.req(8)?;
        Ok(match attrs.endian {
            Endian::Big => buf.get_f64(),
            Endian::Little => buf.get_f64_le(),
        })
    }
}

impl<T> BinSerialize for &T
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        (*self).encode_to(buf, attrs)
    }
}

impl BinSerialize for &str {
    fn encode_to(&self, buf: &mut dyn BufMut, _attrs: Attrs) -> Result<()> {
        buf.put_slice(self.as_bytes());
        buf.put_u8(0);
        Ok(())
    }
}
impl BinSerialize for String {
    fn encode_to(&self, buf: &mut dyn BufMut, _attrs: Attrs) -> Result<()> {
        buf.put_slice(self.as_bytes());
        buf.put_u8(0);
        Ok(())
    }
}
impl BinDeserialize for String {
    fn decode_from(buf: &mut dyn Buf, _attrs: Attrs) -> Result<Self> {
        let mut data = vec![];
        for _ in 0..buf.remaining() {
            match buf.get_u8() {
                0 => return Ok(String::from_utf8(data)?),
                v => data.push(v),
            }
        }
        Err(BinError::InsufficientData(0))
    }
}

impl<T> BinSerialize for Vec<T>
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BufMut, attrs: Attrs) -> Result<()> {
        attrs.encode_length(buf, self.len())?;
        for elem in self {
            elem.encode_to(buf, attrs)?;
        }
        Ok(())
    }
}
impl<T> BinDeserialize for Vec<T>
where
    T: BinDeserialize,
{
    fn decode_from(buf: &mut dyn Buf, attrs: Attrs) -> Result<Self> {
        let len = attrs.decode_length(buf)?;
        let mut v = vec![];
        if let Some(len) = len {
            for _ in 0..len {
                v.push(T::decode_from(buf, attrs)?);
            }
        } else {
            #[allow(clippy::while_let_loop)]
            loop {
                match T::decode_from(buf, attrs) {
                    Ok(elem) => v.push(elem),
                    Err(BinError::InsufficientData(_)) => break,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(v)
    }
}
