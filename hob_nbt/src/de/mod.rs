mod binary_format;
pub mod error;

use self::{binary_format::BinaryFormat, error::DeserializeError};
use crate::nbt_tag::NBTTag;
use proto_bytes::{BytesMut, ConditionalReader};
use serde::{de, forward_to_deserialize_any};
use std::marker::PhantomData;

pub struct Deserializer<B>
where
    B: BinaryFormat,
{
    pub input: BytesMut,
    _marker: PhantomData<B>,
}

impl<B> Deserializer<B>
where
    B: BinaryFormat,
{
    pub fn from_slice(buf: &[u8]) -> Self {
        Deserializer {
            input: BytesMut::from(buf),
            _marker: PhantomData,
        }
    }
    fn eat_value(&mut self, types: NBTTag) {
        use NBTTag::*;
        match types {
            Void => {}
            Byte => B::eat_byte(&mut self.input),
            Short => B::eat_short(&mut self.input),
            Int => B::eat_int(&mut self.input),
            Long => B::eat_long(&mut self.input),
            Float => B::eat_float(&mut self.input),
            Double => B::eat_double(&mut self.input),
            ByteArray => B::eat_byte_array(&mut self.input),
            String => B::eat_string(&mut self.input),
            List => {
                let elem_types = NBTTag::from_i8(B::get_byte(&mut self.input)).unwrap();
                let len = B::get_int(&mut self.input);
                for _ in 0..len {
                    self.eat_value(elem_types)
                }
            }
            Compound => loop {
                let id = B::get_byte(&mut self.input);
                if id == 0 {
                    break;
                }
                self.eat_value(String);
                self.eat_value(NBTTag::from_i8(id).unwrap());
            },
            IntArray => B::eat_int_array(&mut self.input),
            LongArray => B::eat_long_array(&mut self.input),
        }
    }
}

impl<'de, B> de::Deserializer<'de> for &mut Deserializer<B>
where
    B: BinaryFormat,
{
    type Error = DeserializeError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(DeserializeError::Unsupported("Unsupported Type".into()))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let tag = NBTTag::from_i8(B::get_byte(&mut self.input)).unwrap();
        let _ = B::get_string(&mut self.input);
        match tag {
            NBTTag::Compound => {
                let variant = &mut Variant {
                    de: &mut *self,
                    tag,
                };
                variant.deserialize_any(visitor)
            }
            _ => Err(DeserializeError::Message(
                "Not starting with a Tag_Compound".into(),
            )),
        }
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let tag = NBTTag::from_i8(B::get_byte(&mut self.input)).unwrap();
        let _ = B::get_string(&mut self.input);
        match tag {
            NBTTag::List => {
                let variant = &mut Variant {
                    de: &mut *self,
                    tag,
                };
                variant.deserialize_any(visitor)
            }
            _ => Err(DeserializeError::Message(
                "Not starting with a Tag_List".into(),
            )),
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bool
        bytes byte_buf option unit unit_struct newtype_struct tuple identifier
        tuple_struct enum ignored_any
    }
}

struct Variant<'a, B>
where
    B: BinaryFormat,
{
    de: &'a mut Deserializer<B>,
    tag: NBTTag,
}
impl<'de, 'a, B> de::Deserializer<'de> for &mut Variant<'a, B>
where
    B: BinaryFormat,
{
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        use NBTTag::*;
        match self.tag {
            Byte => visitor.visit_i8(B::get_byte(&mut self.de.input)),
            Short => visitor.visit_i16(B::get_short(&mut self.de.input)),
            Int => visitor.visit_i32(B::get_int(&mut self.de.input)),
            Long => visitor.visit_i64(B::get_long(&mut self.de.input)),
            Float => visitor.visit_f32(B::get_float(&mut self.de.input)),
            Double => visitor.visit_f64(B::get_double(&mut self.de.input)),
            String => visitor.visit_string(B::get_string(&mut self.de.input)),
            List => {
                let elem_tag = NBTTag::from_i8(B::get_byte(&mut self.de.input)).unwrap();
                let len = B::get_int(&mut self.de.input);
                visitor.visit_seq(SeqX {
                    de: &mut *self.de,
                    tag: elem_tag,
                    len: len as usize,
                })
            }
            ByteArray | IntArray | LongArray => {
                let len = B::get_int(&mut self.de.input);
                visitor.visit_seq(NumSeqX {
                    de: &mut *self.de,
                    tag: self.tag,
                    len: len as usize,
                })
            }
            Compound => visitor.visit_map(MapX {
                de: &mut *self.de,
                next_tag: NBTTag::Void,
            }),
            Void => Err(DeserializeError::Message("Parse Error".into())),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.eat_value(self.tag);
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.tag {
            NBTTag::Byte => visitor.visit_bool(self.de.input.get_bool()),
            v => Err(DeserializeError::Message(format!(
                "Expected Tag_Byte, Found Tag_{:?}",
                v
            ))),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option struct unit unit_struct tuple seq identifier
        tuple_struct map enum
    }
}

struct MapX<'a, B>
where
    B: BinaryFormat,
{
    de: &'a mut Deserializer<B>,
    next_tag: NBTTag,
}
impl<'de, 'a, B> de::MapAccess<'de> for MapX<'a, B>
where
    B: BinaryFormat,
{
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.next_tag = NBTTag::from_i8(B::get_byte(&mut self.de.input)).unwrap();
        match self.next_tag {
            NBTTag::Void => Ok(None),
            _ => seed
                .deserialize(&mut Variant {
                    de: &mut *self.de,
                    tag: NBTTag::String,
                })
                .map(Some),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Variant {
            de: &mut *self.de,
            tag: self.next_tag,
        })
    }
}

struct SeqX<'a, B>
where
    B: BinaryFormat,
{
    de: &'a mut Deserializer<B>,
    tag: NBTTag,
    len: usize,
}
impl<'de, 'a, B> de::SeqAccess<'de> for SeqX<'a, B>
where
    B: BinaryFormat,
{
    type Error = DeserializeError;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>, Self::Error>
    where
        E: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut Variant {
            de: &mut *self.de,
            tag: self.tag,
        })
        .map(Some)
    }
}

struct NumSeqX<'a, B>
where
    B: BinaryFormat,
{
    de: &'a mut Deserializer<B>,
    tag: NBTTag,
    len: usize,
}
impl<'de, 'a, B> de::SeqAccess<'de> for NumSeqX<'a, B>
where
    B: BinaryFormat,
{
    type Error = DeserializeError;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>, Self::Error>
    where
        E: de::DeserializeSeed<'de>,
    {
        struct NumArrayDeserializer<'a, T>
        where
            T: BinaryFormat,
        {
            de: &'a mut Deserializer<T>,
            types: NBTTag,
        }
        impl<'de, 'a, T> de::Deserializer<'de> for &mut NumArrayDeserializer<'a, T>
        where
            T: BinaryFormat,
        {
            type Error = DeserializeError;
            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                use NBTTag::*;
                match self.types {
                    ByteArray => visitor.visit_i8(T::get_byte_array_elem(&mut self.de.input)),
                    IntArray => visitor.visit_i32(T::get_int_array_elem(&mut self.de.input)),
                    LongArray => visitor.visit_i64(T::get_long_array_elem(&mut self.de.input)),
                    _ => Err(DeserializeError::Message("Parse Error".into())),
                }
            }

            forward_to_deserialize_any! {
                i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bool
                bytes byte_buf option unit unit_struct newtype_struct struct seq tuple identifier
                tuple_struct map enum ignored_any
            }
        }

        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut NumArrayDeserializer {
            de: &mut *self.de,
            types: self.tag,
        })
        .map(Some)
    }
}
