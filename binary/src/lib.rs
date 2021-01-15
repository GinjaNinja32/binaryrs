mod de;
pub use de::{decode_from_bytes, BinDeserialize};

mod ser;
pub use ser::{encode_to_bytes, BinSerialize};

pub mod attr;

pub mod error;
pub use error::{BinError, Result};

mod impls;

pub use binary_derive::{BinDeserialize, BinSerialize};

pub use bytes::BufMut;
pub trait Buf: bytes::Buf {
    fn req(&self, len: usize) -> Result<()> {
        if self.remaining() < len {
            Err(BinError::InsufficientData(len - self.remaining()))
        } else {
            Ok(())
        }
    }
}
impl<T: bytes::Buf> Buf for T {}

pub trait BinFlags {
    fn zero() -> Self;
    fn has(&self, v: u64) -> bool;
    fn set(&mut self, v: u64);
}

// DeOption helper, for binary_derive to un-Option-ify types for decoding when using flags
pub trait DeOption: private::Sealed {
    type Assoc;
}
impl<T> DeOption for Option<T> {
    type Assoc = T;
}
mod private {
    pub trait Sealed {}

    impl<T> Sealed for Option<T> {}
}
