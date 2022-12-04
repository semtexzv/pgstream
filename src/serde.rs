use std::fmt::{Debug, Display, Formatter};

use serde::de::value::MapAccessDeserializer;
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Deserializer};

use crate::decoderbufs::datum_message::Datum;

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct Error(pub anyhow::Error);

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(anyhow::format_err!("{}", msg))
    }
}

impl<'de, 'a: 'de> Deserializer<'de> for &'a Datum {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Datum::DatumBool(b) => visitor.visit_bool(*b),
            Datum::DatumInt32(v) => visitor.visit_i32(*v),
            Datum::DatumInt64(v) => visitor.visit_i64(*v),
            Datum::DatumFloat(v) => visitor.visit_f32(*v),
            Datum::DatumDouble(v) => visitor.visit_f64(*v),
            Datum::DatumString(v) => visitor.visit_str(v),
            Datum::DatumBytes(v) => visitor.visit_borrowed_bytes(v),
            Datum::DatumPoint(_v) => panic!("Not supported"),
            Datum::DatumMissing(_v) => visitor.visit_none(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct Tuple<'a>(&'a [crate::decoderbufs::DatumMessage]);

impl<'de, 'a: 'de> MapAccess<'de> for Tuple<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.0.len() == 0 {
            return Ok(None);
        }
        let name = self.0[0].column_name.as_deref().unwrap();
        seed.deserialize(name.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let (x, rest) = self.0.split_at(1);
        self.0 = rest;
        seed.deserialize(x[0].datum.as_ref().unwrap())
    }
}

pub fn from_row<'de, 'a: 'de, T: Deserialize<'de>>(
    row: &'a [crate::decoderbufs::DatumMessage],
) -> Result<T, Error> {
    let de = MapAccessDeserializer::new(Tuple(row));
    T::deserialize(de)
}

pub mod bytes_array {
    use core::convert::TryInto;

    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    /// This just specializes [`serde_bytes::serialize`] to `<T = [u8]>`.
    pub(crate) fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_bytes::serialize(bytes, serializer)
    }

    /// This takes the result of [`serde_bytes::deserialize`] from `[u8]` to `[u8; N]`.
    pub(crate) fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let slice: &[u8] = serde_bytes::deserialize(deserializer)?;
        let array: [u8; N] = slice.try_into().map_err(|_| {
            let expected = format!("[u8; {}]", N);
            D::Error::invalid_length(slice.len(), &expected.as_str())
        })?;
        Ok(array)
    }
}
