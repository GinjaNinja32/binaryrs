use crate::Result;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone)]
pub struct Attrs {
    pub len: Option<Len>,
    pub len_endian: Endian,
    pub endian: Endian,
}

impl Attrs {
    pub fn zero() -> Self {
        Attrs {
            len: None,
            len_endian: Endian::Little,
            endian: Endian::Little,
        }
    }

    pub fn encode_length(&self, buf: &mut dyn crate::BinWrite, len: u64) -> Result<()> {
        if self.len.is_none() {
            return Ok(());
        }
        self.len.unwrap().encode(len, buf, self.len_endian)
    }

    pub fn decode_length(&self, buf: &mut dyn crate::BinRead) -> Result<Option<u64>> {
        if self.len.is_none() {
            return Ok(None);
        }
        self.len.unwrap().decode(buf, self.len_endian).map(Some)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Len {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl Len {
    pub fn encode(&self, v: u64, buf: &mut dyn crate::BinWrite, endian: Endian) -> Result<()> {
        match endian {
            Endian::Big => match self {
                Len::U8 => buf.put_u8(v.try_into()?),
                Len::U16 => buf.put_u16_be(v.try_into()?),
                Len::U32 => buf.put_u32_be(v.try_into()?),
                Len::U64 => buf.put_u64_be(v),
                Len::I8 => buf.put_i8(v.try_into()?),
                Len::I16 => buf.put_i16_be(v.try_into()?),
                Len::I32 => buf.put_i32_be(v.try_into()?),
                Len::I64 => buf.put_i64_be(v.try_into()?),
            },
            Endian::Little => match self {
                Len::U8 => buf.put_u8(v.try_into()?),
                Len::U16 => buf.put_u16_le(v.try_into()?),
                Len::U32 => buf.put_u32_le(v.try_into()?),
                Len::U64 => buf.put_u64_le(v),
                Len::I8 => buf.put_i8(v.try_into()?),
                Len::I16 => buf.put_i16_le(v.try_into()?),
                Len::I32 => buf.put_i32_le(v.try_into()?),
                Len::I64 => buf.put_i64_le(v.try_into()?),
            },
        }
    }
    pub fn decode(&self, buf: &mut dyn crate::BinRead, endian: Endian) -> Result<u64> {
        let v = match endian {
            Endian::Big => match self {
                Len::U8 => buf.get_u8()? as u64,
                Len::U16 => buf.get_u16_be()? as u64,
                Len::U32 => buf.get_u32_be()? as u64,
                Len::U64 => buf.get_u64_be()?,
                Len::I8 => buf.get_i8()?.try_into()?,
                Len::I16 => buf.get_i16_be()?.try_into()?,
                Len::I32 => buf.get_i32_be()?.try_into()?,
                Len::I64 => buf.get_i64_be()?.try_into()?,
            },
            Endian::Little => match self {
                Len::U8 => buf.get_u8()? as u64,
                Len::U16 => buf.get_u16_le()? as u64,
                Len::U32 => buf.get_u32_le()? as u64,
                Len::U64 => buf.get_u64_le()?,
                Len::I8 => buf.get_i8()?.try_into()?,
                Len::I16 => buf.get_i16_le()?.try_into()?,
                Len::I32 => buf.get_i32_le()?.try_into()?,
                Len::I64 => buf.get_i64_le()?.try_into()?,
            },
        };
        Ok(v)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Endian {
    Little,
    Big,
}
