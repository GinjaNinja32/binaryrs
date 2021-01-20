use crate::Result;

macro_rules! get_stdnum_be {
    ($name:ident,$ty:ty) => {
        fn $name(&mut self) -> Result<$ty> {
            let mut data = [0u8; std::mem::size_of::<$ty>()];
            self.read_exact(&mut data)?;
            Ok(<$ty>::from_be_bytes(data))
        }
    };
}
macro_rules! get_stdnum_le {
    ($name:ident,$ty:ty) => {
        fn $name(&mut self) -> Result<$ty> {
            let mut data = [0u8; std::mem::size_of::<$ty>()];
            self.read_exact(&mut data)?;
            Ok(<$ty>::from_le_bytes(data))
        }
    };
}

pub trait BinRead: std::io::BufRead {
    get_stdnum_be!(get_f32_be, f32);
    get_stdnum_le!(get_f32_le, f32);
    get_stdnum_be!(get_f64_be, f64);
    get_stdnum_le!(get_f64_le, f64);
    get_stdnum_be!(get_i8, i8);
    get_stdnum_be!(get_i16_be, i16);
    get_stdnum_le!(get_i16_le, i16);
    get_stdnum_be!(get_i32_be, i32);
    get_stdnum_le!(get_i32_le, i32);
    get_stdnum_be!(get_i64_be, i64);
    get_stdnum_le!(get_i64_le, i64);
    get_stdnum_be!(get_u8, u8);
    get_stdnum_be!(get_u16_be, u16);
    get_stdnum_le!(get_u16_le, u16);
    get_stdnum_be!(get_u32_be, u32);
    get_stdnum_le!(get_u32_le, u32);
    get_stdnum_be!(get_u64_be, u64);
    get_stdnum_le!(get_u64_le, u64);
}
impl<T: std::io::BufRead> BinRead for T {}

macro_rules! put_stdnum_be {
    ($name:ident,$ty:ty) => {
        fn $name(&mut self, v: $ty) -> Result<()> {
            let data = v.to_be_bytes();
            Ok(self.write_all(&data)?)
        }
    };
}
macro_rules! put_stdnum_le {
    ($name:ident,$ty:ty) => {
        fn $name(&mut self, v: $ty) -> Result<()> {
            let data = v.to_le_bytes();
            Ok(self.write_all(&data)?)
        }
    };
}

pub trait BinWrite: std::io::Write {
    put_stdnum_be!(put_f32_be, f32);
    put_stdnum_le!(put_f32_le, f32);
    put_stdnum_be!(put_f64_be, f64);
    put_stdnum_le!(put_f64_le, f64);
    put_stdnum_be!(put_i8, i8);
    put_stdnum_be!(put_i16_be, i16);
    put_stdnum_le!(put_i16_le, i16);
    put_stdnum_be!(put_i32_be, i32);
    put_stdnum_le!(put_i32_le, i32);
    put_stdnum_be!(put_i64_be, i64);
    put_stdnum_le!(put_i64_le, i64);
    put_stdnum_be!(put_u8, u8);
    put_stdnum_be!(put_u16_be, u16);
    put_stdnum_le!(put_u16_le, u16);
    put_stdnum_be!(put_u32_be, u32);
    put_stdnum_le!(put_u32_le, u32);
    put_stdnum_be!(put_u64_be, u64);
    put_stdnum_le!(put_u64_le, u64);
}
impl<T: std::io::Write> BinWrite for T {}
