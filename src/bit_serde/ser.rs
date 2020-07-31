use bitvec::prelude::*;
use serde::{ser, Serialize};

use super::error::{Error, Result};

pub struct Serializer {
    /// I did say really tight
    buffer: BitVec,
}

pub fn to_bits<T>(value: &T) -> Result<BitVec>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        buffer: BitVec::new(),
    };

    value.serialize(&mut serializer)?;
    Ok(serializer.buffer)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    /// 0 = false, 1 = true
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.buffer.push(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    /// Representation is some number of 1's, followed by a 0, followed by some bits.
    /// deserialize(1{n} 0 bit{n}) = (1 << n) - 1 + bit{n}
    /// where bit{n} is in LSB-MSB order
    fn serialize_u64(self, v: u64) -> Result<()> {
        if v == 0 {
            self.buffer.push(false);
            return Ok(());
        }

        let mut ones = 63 - v.leading_zeros() as u64;
        // u128 cast to avoid overflow in case there are no leading 0's
        if ((1u128 << (ones + 1)) - 1) as u64 == v {
            ones += 1;
        }

        let v = v - ((1u128 << ones) - 1) as u64;
        self.buffer.append(&mut bitvec![1; ones as usize]);
        self.buffer.push(false);
        // wasm is 32-bit and u64::bits does not exist.
        self.buffer
            .extend_from_slice(&(v as u32).bits::<Lsb0>()[..(ones as usize).min(32)]);
        if ones > 32 {
            self.buffer
                .extend_from_slice(&((v >> 32) as u32).bits::<Lsb0>()[..(ones as usize - 32)]);
        }
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    /// Sign bit (0 = nonnegative, 1 = negative),
    /// followed by value if positive and -value - 1 if negative
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.buffer.push(v < 0);
        (if v < 0 { !v } else { v } as u64).serialize(self)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::Unsupported("f32".to_string()))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::Unsupported("f64".to_string()))
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::Unsupported("char".to_string()))
    }

    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(Error::Unsupported("str".to_string()))
    }

    /// First the length, then the elements are
    /// stored in order, each byte in LSB-MSB order
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::Unsupported("bytes".to_string()))
        //v.len().serialize(&mut *self)?;
        //self.buffer.extend(v.iter().flat_map(|b| b.bits::<Lsb0>().iter().copied()));
        //Ok(())
    }

    /// 0 to represent no value
    fn serialize_none(self) -> Result<()> {
        self.buffer.push(false);
        Ok(())
    }

    /// 1 to represent existing value,
    /// followed by value
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.buffer.push(true);
        value.serialize(self)
    }

    /// Units are zero-sized types
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    /// Just the index. It would be nice to know how big the enum is
    /// to know the exact number of bits to use, but alas.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        variant_index.serialize(self)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    /// Variant index, then value. Enum size, please?
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        variant_index.serialize(&mut *self)?;
        value.serialize(self)
    }

    /// Known lengths only for now.
    /// Stores the length, then each element in order.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            len.serialize(&mut *self)?;
            Ok(self)
        } else {
            Err(Error::Unsupported("seq of unknown length".to_string()))
        }
    }

    /// The length is constant because this is a tuple.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    /// Variant index, followed by tuple
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        variant_index.serialize(&mut *self)?;
        self.serialize_tuple(len)
    }

    /// Maps are [k, v, k, v, ...] sequences
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if let Some(len) = len {
            self.serialize_seq(Some(len))
        } else {
            Err(Error::Unsupported("map of unknown length".to_string()))
        }
    }

    /// A struct is a tuple
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }

    /// Variant index followed by struct
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        variant_index.serialize(&mut *self)?;
        self.serialize_struct(name, len)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    /// Length is known; nothing to do here
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(to_bits(&false).unwrap(), bitvec![0]);
        assert_eq!(to_bits(&true).unwrap(), bitvec![1]);
    }

    #[test]
    fn test_uint() {
        assert_eq!(to_bits(&0u32).unwrap(), bitvec![0]);
        assert_eq!(to_bits(&1u32).unwrap(), bitvec![1, 0, 0]);
        assert_eq!(to_bits(&2u32).unwrap(), bitvec![1, 0, 1]);
        assert_eq!(to_bits(&3u32).unwrap(), bitvec![1, 1, 0, 0, 0]);
        assert_eq!(to_bits(&12u32).unwrap(), bitvec![1, 1, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_int() {
        assert_eq!(to_bits(&0i32).unwrap(), bitvec![0, 0]);
        assert_eq!(to_bits(&1i32).unwrap(), bitvec![0, 1, 0, 0]);
        assert_eq!(to_bits(&-3i32).unwrap(), bitvec![1, 1, 0, 1]);
    }

    #[test]
    fn test_option() {
        assert_eq!(to_bits(&(None as Option<u64>)).unwrap(), bitvec![0]);
        assert_eq!(to_bits(&Some(4u32)).unwrap(), bitvec![1, 1, 1, 0, 1, 0]);
    }

    #[test]
    fn test_sequence() {
        assert_eq!(to_bits(&(vec![] as Vec<u64>)).unwrap(), bitvec![0]);
        assert_eq!(
            to_bits(&vec![0u32, 3u32]).unwrap(),
            bitvec![1, 0, 1, 0, 1, 1, 0, 0, 0]
        );
    }

    #[test]
    fn test_tuple() {
        assert_eq!(to_bits(&()).unwrap(), bitvec![]);
        assert_eq!(to_bits(&(0u32, 3u32)).unwrap(), bitvec![0, 1, 1, 0, 0, 0]);
    }
}
