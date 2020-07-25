use serde::Deserialize;
use serde::de::{self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess};
use serde::de::{SeqAccess, VariantAccess, Visitor};
use bitvec::prelude::*;

use super::error::{Error, Result};

pub struct Deserializer<'de> {
    input: &'de BitSlice<Local, usize>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bits(input: &'de BitSlice<Local, usize>) -> Self {
        Deserializer { input }
    }
}

pub fn from_bits<'a, T>(bits: &'a BitSlice<Local, usize>) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bits(bits);
    let t = T::deserialize(&mut deserializer)?;

    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn parse_bool(&mut self) -> Result<bool> {
        let res = *(self.input.first().ok_or(Error::Eof)?);
        self.input = &self.input[1..];
        Ok(res)
    }

    fn parse_uint(&mut self) -> Result<u64> {
        let ones = self.input.iter().position(|b| !*b).ok_or(Error::Eof)?;
        if self.input.len() < 2 * ones + 1 { // too small to fit number
            return Err(Error::Eof);
        }

        let res = if ones == 0 {
            // load_le panics on a 0-element slice
            0
        } else {
            self.input[(ones + 1)..(2 * ones + 1)].load_le::<u64>()
        };

        self.input = &self.input[(2 * ones + 1)..];
        Ok(res + ((1u128 << ones) - 1) as u64)
    }

    fn parse_int(&mut self) -> Result<i64> {
        let neg = self.parse_bool()?;
        let abs = self.parse_uint()?;

        Ok(if neg {
            !(abs as i64)
        } else {
            abs as i64
        })
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_u8(self.parse_uint()? as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_u16(self.parse_uint()? as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_u32(self.parse_uint()? as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_u64(self.parse_uint()? as u64)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_i8(self.parse_int()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_i16(self.parse_int()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_i32(self.parse_int()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_i64(self.parse_int()? as i64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("f32".to_string()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("f64".to_string()))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("char".to_string()))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("str".to_string()))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("string".to_string()))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("bytes".to_string()))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("byte_buf".to_string()))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        // Remember, initial 1 means Some and initial 0 means None
        if self.parse_bool()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        // Length first, then elements
        let len = self.parse_uint()?;
        visitor.visit_seq(Len::new(self, len as usize))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        // No length-parsing necessary
        visitor.visit_seq(Len::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            len: usize,
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        let len = self.parse_uint()?;
        visitor.visit_map(Len::new(self, len as usize))
    }

    fn deserialize_struct<V>(
            self,
            name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_seq(Len::new(self, fields.len()))
    }

    fn deserialize_enum<V>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("identifier".to_string()))
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("deserializing any".to_string()))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        Err(Error::Unsupported("deserializing any".to_string()))
    }
}

/// For reading a sequence of elements with a known length
struct Len<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> Len<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Len { de, len }
    }
}

impl<'a, 'de> SeqAccess<'de> for Len<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

impl<'a, 'de> MapAccess<'de> for Len<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
    }
}

impl<'a, 'de> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>
    {
        let val = seed.deserialize(&mut *self)?;
        Ok((val, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        use serde::Deserializer;
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
            self,
            fields: &'static [&'static str],
            visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>
    {
        use serde::Deserializer;
        self.deserialize_tuple(fields.len(), visitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(from_bits::<bool>(bits![0]).unwrap(), false);
        assert_eq!(from_bits::<bool>(bits![1]).unwrap(), true);
    }

    #[test]
    fn test_uint() {
        assert_eq!(from_bits::<u64>(bits![0]).unwrap(), 0u64);
        assert_eq!(from_bits::<u64>(bits![1,0,0]).unwrap(), 1u64);
        assert_eq!(from_bits::<u64>(bits![1,0,1]).unwrap(), 2u64);
        assert_eq!(from_bits::<u64>(bits![1,1,0,0,0]).unwrap(), 3u64);
        assert_eq!(from_bits::<u64>(bits![1,1,1,1,0,0,1,0,0]).unwrap(), 17u64);
    }

    #[test]
    fn test_int() {
        assert_eq!(from_bits::<i64>(bits![0,0]).unwrap(), 0i64);
        assert_eq!(from_bits::<i64>(bits![0,1,0,1]).unwrap(), 2i64);
        assert_eq!(from_bits::<i64>(bits![1,1,0,0]).unwrap(), -2i64);
    }

    #[test]
    fn test_option() {
        assert_eq!(from_bits::<Option<u64>>(bits![0]).unwrap(), None);
        assert_eq!(from_bits::<Option<u64>>(bits![1,1,0,0]).unwrap(), Some(1));
    }

    #[test]
    fn test_seq() {
        assert_eq!(from_bits::<Vec<u64>>(bits![0]).unwrap(), vec![]);
        assert_eq!(from_bits::<Vec<u64>>(bits![1,1,0,0,0, 1,1,0,1,1, 1,0,1, 0]).unwrap(), vec![6u64, 2, 0]);
    }

    #[test]
    fn test_tuple() {
        assert_eq!(from_bits::<()>(bits![]).unwrap(), ());
        assert_eq!(from_bits::<(u64, u64, i64)>(bits![1,1,0,1,1, 1,0,1, 0,0]).unwrap(), (6u64, 2u64, 0i64));
    }
}