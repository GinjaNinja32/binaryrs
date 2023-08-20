mod de;
pub use de::{decode_from_bytes, decode_from_stream, BinDeserialize};

mod ser;
pub use ser::{encode_to_bytes, encode_to_stream, BinSerialize};

pub mod attr;

pub mod error;
pub use error::{BinError, Result};

mod impls;

mod stream_rw;
pub use stream_rw::{BinRead, BinWrite};

pub use binary_derive::{BinDeserialize, BinSerialize};

pub trait BinFlags {
    const ZERO: Self;
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
