use serde::ser::{Error as _, Impossible, SerializeMap};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::fmt::{Debug, Display, Formatter};

/// Serializer that filters out keys from `T` that collide with [`Doc`]
///
/// [`Doc`]: crate::services::core::Doc
#[derive(Deserialize)]
pub struct Data<T> {
    #[serde(flatten)]
    pub value: T,
}

impl<T> Data<T> {
    const IGNORED_KEYS: &'static [&'static str] = &[
        "_id",
        "space",
        "modifiedOn",
        "modifiedBy",
        "createdBy",
        "createdOn",
        "_class",
    ];

    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Serialize> Serialize for Data<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ser = DataSerializer {
            ignored_keys: Self::IGNORED_KEYS,
        };

        let filtered = self.value.serialize(ser).map_err(S::Error::custom)?;
        filtered.serialize(serializer)
    }
}

impl<T> Debug for Data<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

struct DataSerializer {
    ignored_keys: &'static [&'static str],
}

impl Serializer for DataSerializer {
    type Ok = <serde_json::value::Serializer as Serializer>::Ok;
    type Error = <serde_json::value::Serializer as Serializer>::Error;
    type SerializeSeq = <serde_json::value::Serializer as Serializer>::SerializeSeq;
    type SerializeTuple = <serde_json::value::Serializer as Serializer>::SerializeTuple;
    type SerializeTupleStruct = <serde_json::value::Serializer as Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant =
        <serde_json::value::Serializer as Serializer>::SerializeTupleVariant;
    type SerializeMap = FilteredSerializeMap;
    type SerializeStruct = FilteredSerializeMap;
    type SerializeStructVariant =
        <serde_json::value::Serializer as Serializer>::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_none()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde_json::value::Serializer.serialize_some(value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        serde_json::value::Serializer.serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde_json::value::Serializer.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        serde_json::value::Serializer.serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        serde_json::value::Serializer.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        serde_json::value::Serializer.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        serde_json::value::Serializer.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        serde_json::value::Serializer.serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let ser = serde_json::value::Serializer.serialize_map(len)?;
        Ok(Self::SerializeMap::new(ser, self.ignored_keys))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let ser = serde_json::value::Serializer.serialize_struct(name, len)?;
        Ok(Self::SerializeMap::new(ser, self.ignored_keys))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        serde_json::value::Serializer.serialize_struct_variant(name, variant_index, variant, len)
    }
}

struct FilteredSerializeMap {
    ser: <serde_json::value::Serializer as Serializer>::SerializeMap,
    ignored_keys: &'static [&'static str],
    next_key_ignored: bool,
}

impl FilteredSerializeMap {
    fn new(
        ser: <serde_json::value::Serializer as Serializer>::SerializeMap,
        ignored_keys: &'static [&'static str],
    ) -> Self {
        Self {
            ser,
            ignored_keys,
            next_key_ignored: false,
        }
    }
}

impl serde::ser::SerializeStruct for FilteredSerializeMap {
    type Ok = <<serde_json::value::Serializer as Serializer>::SerializeMap as SerializeMap>::Ok;
    type Error =
        <<serde_json::value::Serializer as Serializer>::SerializeMap as SerializeMap>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeMap>::end(self)
    }
}

impl SerializeMap for FilteredSerializeMap {
    type Ok = <<serde_json::value::Serializer as Serializer>::SerializeMap as SerializeMap>::Ok;
    type Error =
        <<serde_json::value::Serializer as Serializer>::SerializeMap as SerializeMap>::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match key.serialize(MapKeySerializer {
            ignored_keys: self.ignored_keys,
        }) {
            Ok(key) => self.ser.serialize_key(&key),
            Err(MapKeyError::KeyIgnored) => {
                self.next_key_ignored = true;
                Ok(())
            }
            Err(MapKeyError::Json(e)) => Err(e),
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.next_key_ignored {
            self.next_key_ignored = false;
            return Ok(());
        }

        self.ser.serialize_value(value)
    }

    fn end(self) -> Result<Value, Self::Error> {
        self.ser.end()
    }
}

struct MapKeySerializer<'a> {
    ignored_keys: &'a [&'a str],
}

fn key_must_be_a_string() -> MapKeyError {
    MapKeyError::Json(serde_json::Error::custom("key must be a string"))
}

fn float_key_must_be_finite() -> MapKeyError {
    MapKeyError::Json(serde_json::Error::custom(
        "float key must be finite (got NaN or +/-inf)",
    ))
}

#[derive(Debug)]
enum MapKeyError {
    Json(serde_json::Error),
    KeyIgnored,
}

impl core::error::Error for MapKeyError {}

impl Display for MapKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MapKeyError::Json(err) => write!(f, "{err}"),
            MapKeyError::KeyIgnored => write!(f, "KeyIgnored"),
        }
    }
}

impl serde::ser::Error for MapKeyError {
    fn custom<T: Display>(msg: T) -> Self {
        MapKeyError::Json(serde_json::Error::custom(msg))
    }
}

impl<'a> Serializer for MapKeySerializer<'a> {
    type Ok = String;

    type Error = MapKeyError;

    type SerializeSeq = Impossible<String, Self::Error>;

    type SerializeTuple = Impossible<String, Self::Error>;

    type SerializeTupleStruct = Impossible<String, Self::Error>;

    type SerializeTupleVariant = Impossible<String, Self::Error>;

    type SerializeMap = Impossible<String, Self::Error>;

    type SerializeStruct = Impossible<String, Self::Error>;

    type SerializeStructVariant = Impossible<String, Self::Error>;

    fn serialize_bool(self, value: bool) -> Result<String, Self::Error> {
        Ok(if value { "true" } else { "false" }.to_owned())
    }

    fn serialize_i8(self, value: i8) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i16(self, value: i16) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i32(self, value: i32) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i64(self, value: i64) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i128(self, value: i128) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u8(self, value: u8) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u16(self, value: u16) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u32(self, value: u32) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u64(self, value: u64) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u128(self, value: u128) -> Result<String, Self::Error> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_f32(self, value: f32) -> Result<String, Self::Error> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String, Self::Error> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String, Self::Error> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String, Self::Error> {
        if self.ignored_keys.contains(&value) {
            Err(MapKeyError::KeyIgnored)
        } else {
            Ok(value.to_owned())
        }
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, Self::Error> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}
