use binary::{BinDeserialize, BinSerialize};
use std::convert::TryFrom;

macro_rules! roundtrip {
    ($val:expr, $bytes:expr) => {
        assert_eq!(binary::encode_to_bytes($val), Ok($bytes));
        assert_eq!(binary::decode_from_bytes(&$bytes), Ok($val));
    };
}

#[test]
fn test_primitive() {
    roundtrip!(42u8, vec![42]);
    roundtrip!(-5i8, vec![251]);

    roundtrip!(42u16, vec![42, 0]);
    roundtrip!(-5i16, vec![251, 255]);

    roundtrip!(42u32, vec![42, 0, 0, 0]);
    roundtrip!(-5i32, vec![251, 255, 255, 255]);

    roundtrip!(42u64, vec![42, 0, 0, 0, 0, 0, 0, 0]);
    roundtrip!(-5i64, vec![251, 255, 255, 255, 255, 255, 255, 255]);

    roundtrip!(true, vec![1]);
    roundtrip!(false, vec![0]);

    roundtrip!(2.0f32, vec![0, 0, 0, 64]);
    roundtrip!(2.0f64, vec![0, 0, 0, 0, 0, 0, 0, 64]);
}

#[test]
fn test_str() {
    roundtrip!("".to_string(), vec![0]);

    roundtrip!("test".to_string(), vec![116, 101, 115, 116, 0]);
}

#[test]
fn test_vec() {
    roundtrip!(Vec::<u8>::new(), vec![]);

    roundtrip!(vec![1u8, 2, 3, 4], vec![1, 2, 3, 4]);
    roundtrip!(vec![1u16, 2, 3, 4], vec![1, 0, 2, 0, 3, 0, 4, 0]);
}

#[test]
fn test_struct() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    struct X {
        a: u16,
        b: String,
        #[binary(len(u16))]
        c: Vec<u8>,
        #[binary(len(u32, big))]
        d: Vec<u8>,
        e: u8,
    }

    roundtrip!(
        X {
            a: 10000,
            b: "test".to_string(),
            c: vec![1, 2, 3, 4],
            d: vec![66],
            e: 42,
        },
        vec![16, 39, 116, 101, 115, 116, 0, 4, 0, 1, 2, 3, 4, 0, 0, 0, 1, 66, 42]
    );

    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    struct Y(u16, String, #[binary(len(u16))] Vec<u8>);

    roundtrip!(
        Y(10000, "test".to_string(), vec![1, 2, 3, 4]),
        vec![16, 39, 116, 101, 115, 116, 0, 4, 0, 1, 2, 3, 4]
    );
}

#[test]
fn test_enum_plain() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    #[repr(u8)]
    enum Plain {
        A,
        B = 42,
        C,
    }

    roundtrip!(Plain::A, vec![0]);
    roundtrip!(Plain::B, vec![42]);
    roundtrip!(Plain::C, vec![43]);
}

#[test]
fn test_enum_fields() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    #[repr(u8)]
    enum Fields {
        A,
        B(u8, String),
        C { x: u32, y: u32 },
    }

    roundtrip!(Fields::A, vec![0]);
    roundtrip!(
        Fields::B(42, "test".to_string()),
        vec![1, 42, 116, 101, 115, 116, 0]
    );
    roundtrip!(
        Fields::C { x: 42, y: 100 },
        vec![2, 42, 0, 0, 0, 100, 0, 0, 0]
    );
}

#[test]
fn test_enum_nested() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    #[repr(u8)]
    #[binary(nest(u8))]
    enum Nest {
        A,
        B(u8, String),
        C { x: u32, y: u32 },
    }

    roundtrip!(Nest::A, vec![0, 0]);
    roundtrip!(
        Nest::B(42, "test".to_string()),
        vec![1, 6, 42, 116, 101, 115, 116, 0]
    );
    roundtrip!(
        Nest::C { x: 42, y: 100 },
        vec![2, 8, 42, 0, 0, 0, 100, 0, 0, 0]
    );
}

#[test]
fn test_flags() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    struct Flags {
        #[binary(flags)]
        flags: u8,
        #[binary(flags(0x01))]
        field1: Option<u16>,
        #[binary(flags(0x02))]
        field2: Option<u8>,
        #[binary(flags(0x04))]
        field4: Option<String>,
    }

    roundtrip!(
        Flags {
            flags: 1,
            field1: Some(1000),
            field2: None,
            field4: None
        },
        vec![1, 232, 3]
    );
    roundtrip!(
        Flags {
            flags: 2,
            field1: None,
            field2: Some(42),
            field4: None,
        },
        vec![2, 42]
    );
    roundtrip!(
        Flags {
            flags: 4,
            field1: None,
            field2: None,
            field4: Some("test".to_string()),
        },
        vec![4, 116, 101, 115, 116, 0]
    );
    roundtrip!(
        Flags {
            flags: 7,
            field1: Some(1000),
            field2: Some(42),
            field4: Some("test".to_string()),
        },
        vec![7, 232, 3, 42, 116, 101, 115, 116, 0]
    );
    assert_eq!(
        binary::encode_to_bytes(Flags {
            flags: 0, // encodes as 7 due to Some() fields below
            field1: Some(1000),
            field2: Some(42),
            field4: Some("test".to_string())
        }),
        Ok(vec![7, 232, 3, 42, 116, 101, 115, 116, 0])
    );
}

#[test]
fn test_default_variant() {
    #[derive(BinSerialize, BinDeserialize, Debug, PartialEq, Eq)]
    #[repr(u16)]
    #[binary(tag(big))]
    enum DefaultVariant {
        A(u64),
        B(u32),
        C(u16),
        #[binary(default)]
        Unknown(u16, Vec<u8>),
    }

    roundtrip!(
        DefaultVariant::A(0x1122334455667788),
        vec![0, 0, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]
    );
    roundtrip!(
        DefaultVariant::B(0x11223344),
        vec![0, 1, 0x44, 0x33, 0x22, 0x11]
    );
    roundtrip!(DefaultVariant::C(0x1122), vec![0, 2, 0x22, 0x11]);
    roundtrip!(
        DefaultVariant::Unknown(42, vec![1, 2, 3, 4]),
        vec![0, 42, 1, 2, 3, 4]
    );
}

#[derive(BinSerialize, BinDeserialize, PartialEq, Eq, Debug)]
struct TestStruct {
    #[binary(len(u8))]
    x: Vec<u8>,
}

#[test]
fn test_encode_error() {
    assert_eq!(
        binary::encode_to_bytes(TestStruct { x: vec![0; 256] }),
        Err(binary::BinError::IntTooLarge(
            u8::try_from(256usize).err().unwrap()
        ))
    );
}

#[test]
fn test_decode_error() {
    assert_eq!(
        binary::decode_from_bytes::<TestStruct>(&[3, 1, 2]),
        Err(binary::BinError::InsufficientData(1))
    );
}
