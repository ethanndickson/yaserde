//! Regression tests for `HexBinaryYaSerde`.
//!
//! `HexBinaryYaSerde` serializes bitflag-shaped newtypes as XSD `hexBinary`:
//! uppercase hex digits in whole octets, no prefix, no sign. XSD does not fix the
//! width, so the derive renders the fewest whole octets that hold the value, the
//! way the IEEE 2030.5 spec writes its own examples. The digit count is therefore
//! always even but never padded to the primitive's width. `FromStr` accepts any
//! number of leading zeros and round-trips the rendered form.

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
fn display_uses_fewest_whole_octets() {
    assert_eq!(Hex8(0).to_string(), "00");
    assert_eq!(Hex8(1).to_string(), "01");
    assert_eq!(Hex8(0xFF).to_string(), "FF");

    assert_eq!(Hex16(0).to_string(), "00");
    assert_eq!(Hex16(0x10).to_string(), "10");
    assert_eq!(Hex16(0x100).to_string(), "0100");
    assert_eq!(Hex16(0xABCD).to_string(), "ABCD");

    assert_eq!(Hex32(0).to_string(), "00");
    assert_eq!(Hex32(1).to_string(), "01");
    assert_eq!(Hex32(0xF4240).to_string(), "0F4240");
    assert_eq!(Hex32(0xDEADBEEF).to_string(), "DEADBEEF");

    assert_eq!(Hex64(0).to_string(), "00");
    assert_eq!(Hex64(0xCAFEBABE).to_string(), "CAFEBABE");
    assert_eq!(Hex64(u64::MAX).to_string(), "FFFFFFFFFFFFFFFF");
}

#[test]
fn display_is_never_odd_length() {
    for shift in 0..64 {
        let v = 1u64 << shift;
        for v in [v, v | (v - 1)] {
            let s = Hex64(v).to_string();
            assert_eq!(s.len() % 2, 0, "odd digit count for {v:#x}: {s}");
            assert!(s.len() == 2 || !s.starts_with("00"), "padded: {}", s);
        }
    }
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
        assert_eq!(Hex32::from_str(&s).unwrap(), Hex32(v));
        // Peers may pad to the primitive's full width.
        assert_eq!(Hex32::from_str(&format!("{s:0>8}")).unwrap(), Hex32(v));
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

#[test]
fn fromstr_rejects_sign_and_empty() {
    use std::str::FromStr;
    // `u64::from_str_radix` accepts a leading `+`; hexBinary does not.
    assert!(Hex8::from_str("+1").is_err());
    assert!(Hex8::from_str("-1").is_err());
    assert!(Hex8::from_str("").is_err());
}

#[test]
fn fromstr_accepts_extra_leading_zeros() {
    use std::str::FromStr;
    assert_eq!(Hex8::from_str("0001").unwrap(), Hex8(1));
    assert_eq!(
        Hex8::from_str("00000000000000000000FF").unwrap(),
        Hex8(0xFF)
    );
}
