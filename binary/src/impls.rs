use crate::attr::{Attrs, Endian};
use crate::{BinDeserialize, BinError, BinFlags, BinRead, BinSerialize, BinWrite, Result};
use std::convert::TryInto;

impl BinSerialize for bool {
    fn encode_to(&self, buf: &mut dyn BinWrite, _attrs: Attrs) -> Result<()> {
        buf.put_u8(if *self { 1 } else { 0 })
    }
}
impl BinDeserialize for bool {
    fn decode_from(buf: &mut dyn BinRead, _attrs: Attrs) -> Result<Self> {
        Ok(buf.get_u8()? != 0)
    }
}

impl BinSerialize for i8 {
    fn encode_to(&self, buf: &mut dyn BinWrite, _attrs: Attrs) -> Result<()> {
        buf.put_i8(*self)
    }
}
impl BinDeserialize for i8 {
    fn decode_from(buf: &mut dyn BinRead, _attrs: Attrs) -> Result<Self> {
        buf.get_i8()
    }
}

impl BinSerialize for u8 {
    fn encode_to(&self, buf: &mut dyn BinWrite, _attrs: Attrs) -> Result<()> {
        buf.put_u8(*self)
    }
}
impl BinDeserialize for u8 {
    fn decode_from(buf: &mut dyn BinRead, _attrs: Attrs) -> Result<Self> {
        buf.get_u8()
    }
}
impl BinFlags for u8 {
    fn zero() -> Self {
        0
    }
    fn has(&self, v: u64) -> bool {
        (*self & (v as u8)) != 0
    }
    fn set(&mut self, v: u64) {
        *self |= v as u8
    }
}

impl BinSerialize for u16 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u16_be(*self),
            Endian::Little => buf.put_u16_le(*self),
        }
    }
}
impl BinDeserialize for u16 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_u16_be(),
            Endian::Little => buf.get_u16_le(),
        }
    }
}
impl BinFlags for u16 {
    fn zero() -> Self {
        0
    }
    fn has(&self, v: u64) -> bool {
        (*self & (v as u16)) != 0
    }
    fn set(&mut self, v: u64) {
        *self |= v as u16
    }
}

impl BinSerialize for i16 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i16_be(*self),
            Endian::Little => buf.put_i16_le(*self),
        }
    }
}
impl BinDeserialize for i16 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_i16_be(),
            Endian::Little => buf.get_i16_le(),
        }
    }
}

impl BinSerialize for u32 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u32_be(*self),
            Endian::Little => buf.put_u32_le(*self),
        }
    }
}
impl BinDeserialize for u32 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_u32_be(),
            Endian::Little => buf.get_u32_le(),
        }
    }
}
impl BinFlags for u32 {
    fn zero() -> Self {
        0
    }
    fn has(&self, v: u64) -> bool {
        (*self & (v as u32)) != 0
    }
    fn set(&mut self, v: u64) {
        *self |= v as u32
    }
}

impl BinSerialize for i32 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i32_be(*self),
            Endian::Little => buf.put_i32_le(*self),
        }
    }
}
impl BinDeserialize for i32 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_i32_be(),
            Endian::Little => buf.get_i32_le(),
        }
    }
}

impl BinSerialize for u64 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_u64_be(*self),
            Endian::Little => buf.put_u64_le(*self),
        }
    }
}
impl BinDeserialize for u64 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_u64_be(),
            Endian::Little => buf.get_u64_le(),
        }
    }
}
impl BinFlags for u64 {
    fn zero() -> Self {
        0
    }
    fn has(&self, v: u64) -> bool {
        (*self & v) != 0
    }
    fn set(&mut self, v: u64) {
        *self |= v
    }
}

impl BinSerialize for i64 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_i64_be(*self),
            Endian::Little => buf.put_i64_le(*self),
        }
    }
}
impl BinDeserialize for i64 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_i64_be(),
            Endian::Little => buf.get_i64_le(),
        }
    }
}

impl BinSerialize for f32 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_f32_be(*self),
            Endian::Little => buf.put_f32_le(*self),
        }
    }
}
impl BinDeserialize for f32 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_f32_be(),
            Endian::Little => buf.get_f32_le(),
        }
    }
}

impl BinSerialize for f64 {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        match attrs.endian {
            Endian::Big => buf.put_f64_be(*self),
            Endian::Little => buf.put_f64_le(*self),
        }
    }
}
impl BinDeserialize for f64 {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        match attrs.endian {
            Endian::Big => buf.get_f64_be(),
            Endian::Little => buf.get_f64_le(),
        }
    }
}

impl<T> BinSerialize for &T
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        (*self).encode_to(buf, attrs)
    }
}

impl BinSerialize for &str {
    fn encode_to(&self, buf: &mut dyn BinWrite, _attrs: Attrs) -> Result<()> {
        buf.write_all(self.as_bytes())?;
        buf.put_u8(0)
    }
}
impl BinSerialize for String {
    fn encode_to(&self, buf: &mut dyn BinWrite, _attrs: Attrs) -> Result<()> {
        buf.write_all(self.as_bytes())?;
        buf.put_u8(0)
    }
}
impl BinDeserialize for String {
    fn decode_from(buf: &mut dyn BinRead, _attrs: Attrs) -> Result<Self> {
        let mut data = vec![];
        loop {
            match buf.get_u8()? {
                0 => return Ok(String::from_utf8(data)?),
                v => data.push(v),
            }
        }
    }
}

impl<T> BinSerialize for Vec<T>
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        attrs.encode_length(buf, self.len() as u64)?;
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
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        let len = attrs.decode_length(buf)?;
        let mut v = vec![];
        if let Some(len) = len {
            let len: usize = len.try_into()?; // usize might be u32, so we need to check
            for _ in 0..len {
                v.push(T::decode_from(buf, attrs)?);
            }
        } else {
            #[allow(clippy::while_let_loop)]
            loop {
                match T::decode_from(buf, attrs) {
                    Ok(elem) => v.push(elem),
                    Err(BinError::InsufficientData) => break,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(v)
    }
}

impl<T, const N: usize> BinDeserialize for [T; N]
where
    T: BinDeserialize,
{
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            v.push(T::decode_from(buf, attrs)?);
        }
        Ok(v.try_into().ok().unwrap())
    }
}

impl<T, const N: usize> BinSerialize for [T; N]
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        for item in self {
            item.encode_to(buf, attrs)?;
        }
        Ok(())
    }
}

impl<T> BinDeserialize for Box<T>
where
    T: BinDeserialize,
{
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        Ok(Box::new(T::decode_from(buf, attrs)?))
    }
}
impl<T> BinSerialize for Box<T>
where
    T: BinSerialize,
{
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        T::encode_to(self, buf, attrs)
    }
}

impl BinSerialize for () {
    fn encode_to(&self, buf: &mut dyn BinWrite, attrs: Attrs) -> Result<()> {
        Ok(())
    }
}
impl BinDeserialize for () {
    fn decode_from(buf: &mut dyn BinRead, attrs: Attrs) -> Result<Self> {
        Ok(())
    }
}
