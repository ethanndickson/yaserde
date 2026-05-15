//! Regression tests for `HexBinaryYaSerde`.
//!
//! `HexBinaryYaSerde` is used to serialize bitflag-shaped newtypes as fixed-width
//! uppercase hex strings, matching the XSD `hexBinary` schema type. The width is
//! determined by the underlying primitive (HexBinary8 -> 2 chars, HexBinary16 -> 4,
//! HexBinary32 -> 8, HexBinary64 -> 16), and round-trips through `FromStr` which
//! parses the string as a hex literal.

use sepserde::HexBinaryYaSerde;

macro_rules! hexbinary_newtype {
    ($name:ident, $prim:ty) => {
        #[derive(Default, PartialEq, Eq, Clone, Copy, Debug, HexBinaryYaSerde)]
        pub struct $name(pub $prim);
        impl $name {
            pub fn bits(&self) -> $prim {
                self.0
            }
            pub fn from_bits(v: $prim) -> Option<Self> {
                Some(Self(v))
            }
        }
    };
}

hexbinary_newtype!(Hex8, u8);
hexbinary_newtype!(Hex16, u16);
hexbinary_newtype!(Hex32, u32);
hexbinary_newtype!(Hex64, u64);

#[test]
fn display_width_is_fixed_by_primitive() {
    assert_eq!(Hex8(0).to_string(), "00");
    assert_eq!(Hex8(1).to_string(), "01");
    assert_eq!(Hex8(0xFF).to_string(), "FF");

    assert_eq!(Hex16(0).to_string(), "0000");
    assert_eq!(Hex16(0x10).to_string(), "0010");
    assert_eq!(Hex16(0xABCD).to_string(), "ABCD");

    assert_eq!(Hex32(0).to_string(), "00000000");
    assert_eq!(Hex32(1).to_string(), "00000001");
    assert_eq!(Hex32(0xDEADBEEF).to_string(), "DEADBEEF");

    assert_eq!(Hex64(0).to_string(), "0000000000000000");
    assert_eq!(Hex64(0xCAFEBABE).to_string(), "00000000CAFEBABE");
}

#[test]
fn fromstr_parses_hex() {
    use std::str::FromStr;
    assert_eq!(Hex8::from_str("01").unwrap(), Hex8(1));
    assert_eq!(Hex8::from_str("FF").unwrap(), Hex8(0xFF));
    assert_eq!(Hex32::from_str("DEADBEEF").unwrap(), Hex32(0xDEADBEEF));
    assert_eq!(
        Hex64::from_str("00000000CAFEBABE").unwrap(),
        Hex64(0xCAFEBABE)
    );
}

#[test]
fn round_trip_through_display_and_fromstr() {
    use std::str::FromStr;
    for v in [0u32, 1, 0x10, 0xABCD, 0xDEADBEEF, u32::MAX] {
        let s = Hex32(v).to_string();
        assert_eq!(s.len(), 8, "width regression for {v:#x}");
        assert_eq!(Hex32::from_str(&s).unwrap(), Hex32(v));
    }
}

#[test]
fn fromstr_rejects_overflow() {
    use std::str::FromStr;
    // 0x100 doesn't fit in u8
    assert!(Hex8::from_str("100").is_err());
    // garbage
    assert!(Hex8::from_str("ZZ").is_err());
}
