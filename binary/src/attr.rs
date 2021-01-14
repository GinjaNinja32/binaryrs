use crate::Result;
    use std::convert::TryInto;

    #[derive(Debug, Copy, Clone)]
    pub struct Attrs {
        pub len: Len,
        pub len_endian: Endian,
        pub endian: Endian,
    }

    impl Attrs {
        pub fn zero() -> Self {
            Attrs {
                len: Len::None,
                len_endian: Endian::Little,
                endian: Endian::Little,
            }
        }

        pub fn encode_length(&self, buf: &mut dyn crate::BufMut, len: usize) -> Result<()> {
            match self.len_endian {
                Endian::Big => match self.len {
                    Len::None => {}
                    Len::U8 => buf.put_u8(len.try_into()?),
                    Len::U16 => buf.put_u16(len.try_into()?),
                    Len::U32 => buf.put_u32(len.try_into()?),
                    Len::U64 => buf.put_u64(len.try_into()?),
                },
                Endian::Little => match self.len {
                    Len::None => {}
                    Len::U8 => buf.put_u8(len.try_into()?),
                    Len::U16 => buf.put_u16_le(len.try_into()?),
                    Len::U32 => buf.put_u32_le(len.try_into()?),
                    Len::U64 => buf.put_u64_le(len.try_into()?),
                },
            }
            Ok(())
        }

        pub fn decode_length(&self, buf: &mut dyn crate::Buf) -> Result<Option<usize>> {
            match self.len {
                Len::None => {}
                Len::U8 => buf.req(1)?,
                Len::U16 => buf.req(2)?,
                Len::U32 => buf.req(4)?,
                Len::U64 => buf.req(8)?,
            }
            let v = match self.len_endian {
                Endian::Big => match self.len {
                    Len::None => None,
                    Len::U8 => Some(buf.get_u8() as usize),
                    Len::U16 => Some(buf.get_u16() as usize),
                    Len::U32 => Some(buf.get_u32() as usize),
                    Len::U64 => Some(buf.get_u64() as usize),
                },
                Endian::Little => match self.len {
                    Len::None => None,
                    Len::U8 => Some(buf.get_u8() as usize),
                    Len::U16 => Some(buf.get_u16_le() as usize),
                    Len::U32 => Some(buf.get_u32_le() as usize),
                    Len::U64 => Some(buf.get_u64_le() as usize),
                },
            };
            Ok(v)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Len {
        None,
        U8,
        U16,
        U32,
        U64,
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Endian {
        Little,
        Big,
    }