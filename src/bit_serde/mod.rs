//! This format packs fields really tight,
//! expecting numbers to be very small,
//! to keep URLs short
//!
//! The entire `GadgetDef` struct consists of small numbers,
//! and the port map and current state of `Gadget`
//! also contain small numbers.

mod ser;
mod de;
mod error;

pub use error::{Error, Result};
pub use ser::{to_bits, Serializer};
pub use de::{from_bits, Deserializer};

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;
    use serde::Serialize;
    use serde::Deserialize;
    use serde::de::DeserializeOwned;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum TestEnum {
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
        assert_eq!(from_bits::<T>(&bits).unwrap(), t, "{:?} failed to deserialize to {:?}", bits, t);
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
        round_trip(vec![vec![false, true, false], vec![true, true, true], vec![false, true, false]]);
    }
}