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
        #[binary(len = 2)]
        c: Vec<u8>,
        #[binary(len = 4, len_big)]
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
}

#[derive(BinSerialize, BinDeserialize, PartialEq, Eq, Debug)]
struct TestStruct {
    #[binary(len = 1)]
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
