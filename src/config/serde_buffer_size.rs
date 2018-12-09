use serde::{
    de::{Deserializer, Error, Unexpected, Visitor},
    ser::Serializer,
};
use std::fmt;

pub fn serialize<S>(size: &cpal::BufferSize, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *size {
        cpal::BufferSize::Default => s.serialize_str("default"),
        cpal::BufferSize::Fixed(n) => s.serialize_u32(n as u32),
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<cpal::BufferSize, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_any(BufferSizeVisitor)
}

struct BufferSizeVisitor;

impl<'de> Visitor<'de> for BufferSizeVisitor {
    type Value = cpal::BufferSize;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the string \"default\" or a positive nonzero integer"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v == "default" {
            Ok(cpal::BufferSize::Default)
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(cpal::BufferSize::Fixed(v as usize))
        } else {
            Err(E::invalid_value(Unexpected::Signed(v), &self))
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v > 0 {
            Ok(cpal::BufferSize::Fixed(v as usize))
        } else {
            Err(E::invalid_value(Unexpected::Unsigned(v), &self))
        }
    }
}
