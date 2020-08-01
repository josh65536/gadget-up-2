//! This format packs fields really tight,
//! expecting numbers to be very small,
//! to keep URLs short
//!
//! The entire `GadgetDef` struct consists of small numbers,
//! and the port map and current state of `Gadget`
//! also contain small numbers.

mod de;
mod error;
mod ser;

use bitvec::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use de::{from_bits, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bits, Serializer};

const fn base64_map_inv_() -> [u8; 128] {
    /// Generates the 64 assignments, since for loops aren't allowed
    macro_rules! assign {
        ($map:expr, $inv:expr) => { assign!($map, $inv, 0 b b b b b b)};

        ($map:expr, $inv:expr, $num:tt b $($b:tt)*) => {
            {
                assign!($map, $inv, ($num * 2) $($b)*);
                assign!($map, $inv, ($num * 2 + 1) $($b)*);
            }
        };

        ($map:expr, $inv:expr, $num:tt) => {
            $inv[$map[$num] as usize] = $num;
        }
    }

    let mut inv_map = [0; 128];
    assign!(BASE64_MAP, inv_map);
    inv_map
}

const BASE64_MAP: &'static [u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

const BASE64_INV: &'static [u8] = &base64_map_inv_();

fn bits_to_base64(bitvec: &BitSlice) -> (String, usize) {
    let padding = (-(bitvec.len() as isize)).rem_euclid(6) as usize;
    let base64 = bitvec
        .chunks(6)
        .map(|slice| BASE64_MAP[slice.load_le::<usize>()] as char)
        .collect();
    (base64, padding)
}

fn base64_to_bits(s: &str, padding: usize) -> BitVec {
    let mut bitvec = s
        .chars()
        .flat_map(|c| &BASE64_INV[c as usize].bits::<Lsb0>()[..6])
        .copied()
        .collect::<BitVec>();
    bitvec.truncate(bitvec.len() - padding);
    bitvec
}

/// Helper function to convert a serializable type to a base64 string,
/// along with the number of bits added as padding
pub fn to_base64<T: Serialize>(t: &T) -> Result<(String, usize)> {
    let bitvec = to_bits(t)?;
    Ok(bits_to_base64(&bitvec))
}

/// Helper function to convert a base64 string, along with number of bits of padding, to a deserializable type
pub fn from_base64<T: DeserializeOwned>(s: &str, padding: usize) -> Result<T> {
    let bitvec = base64_to_bits(s, padding);
    from_bits(&bitvec)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use std::fmt::Debug;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Enum {
        Unit,
        Newtype(u64),
        Tuple(u64, i64),
        Struct { a: Option<bool>, b: u64 },
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct UnitStruct;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NewtypeStruct(u64);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TupleStruct(Vec<u64>, i64);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Struct {
        a: Option<()>,
        b: i64,
        c: (Option<Box<Struct>>, u64),
    }

    fn round_trip<T: Debug + Serialize + DeserializeOwned + PartialEq>(t: T) {
        let bits = to_bits(&t).expect(&format!("Failed to serialize {:?}", t));
        assert_eq!(
            from_bits::<T>(&bits).unwrap(),
            t,
            "{:?} failed to deserialize to {:?}",
            bits,
            t
        );

        let (base64, padding) = to_base64(&t).unwrap();
        assert_eq!(from_base64::<T>(&base64, padding).unwrap(), t);
    }

    fn round_trip_bitvec_base64(bits: &BitSlice) {
        let (base64, padding) = bits_to_base64(&bits);
        let result = base64_to_bits(&base64, padding);
        assert_eq!(
            result,
            bits,
            "{:?} converted to {:?} instead of {:?}",
            (base64, padding),
            &result,
            bits,
        );
    }

    #[test]
    fn test_base64_regression_1() {
        // for Struct { a: None, b: 1u64 }
        round_trip_bitvec_base64(bits![0, 1, 0, 0]);
    }

    #[test]
    fn test_bool() {
        round_trip(false);
        round_trip(true);
    }

    #[test]
    fn test_uint() {
        for i in 0u64..256u64 {
            round_trip(i);
        }
        round_trip(std::u64::MAX);
    }

    #[test]
    fn test_int() {
        for i in -128i64..128i64 {
            round_trip(i);
        }
        round_trip(std::i64::MIN);
        round_trip(std::i64::MAX);
    }

    #[test]
    fn test_option() {
        round_trip(None as Option<u64>);
        round_trip(Some(5i64));
        round_trip(Some(()));
        round_trip(Some(None as Option<u64>));
    }

    #[test]
    fn test_seq() {
        round_trip(vec![] as Vec<u64>);
        round_trip(vec![(), (), ()]);
        round_trip(vec![1u64, 2, 8, 4]);
    }

    #[test]
    fn test_seq_nest() {
        round_trip(vec![
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
        ]);
    }

    #[test]
    fn test_tuple() {
        round_trip(());
        round_trip((1u64,));
        round_trip((3u64, 4u64, 7i64, Some(false)));
    }

    #[test]
    fn test_tuple_nest() {
        round_trip(((5i64, 2i64), (None as Option<()>, (), false, true)));
    }

    #[test]
    fn test_map() {
        use fnv::FnvHashMap;
        round_trip(FnvHashMap::<(), ()>::default());
        round_trip(
            [(1u64, false), (7u64, true), (11u64, true)]
                .iter()
                .copied()
                .collect::<FnvHashMap<_, _>>(),
        );
    }

    #[test]
    fn test_enum_unit() {
        round_trip(Enum::Unit);
    }

    #[test]
    fn test_enum_newtype() {
        round_trip(Enum::Newtype(0));
        round_trip(Enum::Newtype(43));
    }

    #[test]
    fn test_enum_tuple() {
        round_trip(Enum::Tuple(0, 0));
        round_trip(Enum::Tuple(8, -1));
    }

    #[test]
    fn test_enum_struct() {
        round_trip(Enum::Struct { a: None, b: 1 });
        round_trip(Enum::Struct {
            a: Some(true),
            b: 8,
        });
    }

    #[test]
    fn test_struct_unit() {
        round_trip(UnitStruct)
    }

    #[test]
    fn test_struct_newtype() {
        round_trip(NewtypeStruct(0));
        round_trip(NewtypeStruct(2));
    }

    #[test]
    fn test_struct_tuple() {
        round_trip(TupleStruct(vec![8, 7, 99], 3));
        round_trip(TupleStruct(vec![], -11));
    }

    #[test]
    fn test_struct() {
        round_trip(Struct {
            a: None,
            b: 69,
            c: (None, 2),
        });
        round_trip(Struct {
            a: Some(()),
            b: -7,
            c: (
                Some(Box::new(Struct {
                    a: Some(()),
                    b: 0,
                    c: (None, 10),
                })),
                0,
            ),
        });
    }
}
