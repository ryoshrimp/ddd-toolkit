// Check Serde protocol methods and type names, which JSON alone cannot observe.
use ddd_toolkit::{Entity, ValueObject};
use serde::{Deserialize, Serialize, Serializer, ser::Impossible};

type Error = serde_json::Error;
struct Probe;
fn observed<T>(method: impl std::fmt::Display) -> Result<T, Error> {
    Err(serde::ser::Error::custom(method))
}
macro_rules! primitives {
    ($($name:ident($($arg:ident: $ty:ty),*));* $(;)?) => {$(
        fn $name(self, $($arg: $ty),*) -> Result<(), Error> {
            observed(stringify!($name))
        }
    )*};
}
impl Serializer for Probe {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;
    primitives! {
        serialize_bool(_v: bool); serialize_i8(_v: i8); serialize_i16(_v: i16);
        serialize_i32(_v: i32); serialize_i64(_v: i64); serialize_u8(_v: u8);
        serialize_u16(_v: u16); serialize_u32(_v: u32); serialize_u64(_v: u64);
        serialize_f32(_v: f32); serialize_f64(_v: f64); serialize_char(_v: char);
        serialize_str(_v: &str); serialize_bytes(_v: &[u8]); serialize_none(); serialize_unit();
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _v: &T) -> Result<(), Error> {
        observed("some")
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        observed(name)
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        observed(format!("unit_variant:{name}:{variant}"))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _v: &T,
    ) -> Result<(), Error> {
        observed(name)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        _v: &T,
    ) -> Result<(), Error> {
        observed(format!("newtype_variant:{name}:{variant}"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        observed("seq")
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        observed(format!("tuple:{len}"))
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        observed(format!("tuple_struct:{name}"))
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        observed(format!("tuple_variant:{name}:{variant}"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        observed("map")
    }
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        observed(format!("struct:{name}"))
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _i: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        observed(format!("struct_variant:{name}:{variant}"))
    }
}

#[derive(ValueObject)]
#[value_object(serde)]
struct Single {
    value: u64,
}
#[derive(ValueObject)]
#[value_object(serde)]
struct Tuple(pub u64, pub u64);
#[derive(ValueObject)]
#[value_object(serde)]
struct Named {
    left: u64,
    right: u64,
}
#[derive(ValueObject)]
#[value_object(serde)]
enum Choice {
    Unit,
    Newtype(u64),
    Tuple(u64, u64),
    Named { value: u64 },
}
#[derive(Entity)]
#[entity(serde)]
struct OnlyId {
    #[entity(id)]
    id: u64,
}
#[derive(Entity)]
#[entity(serde)]
#[serde(rename(deserialize = "Input"))]
struct OutputDefault {
    #[entity(id)]
    id: u64,
}
#[derive(Entity)]
#[entity(serde)]
#[serde(rename(serialize = "Output"))]
struct InputDefault {
    #[entity(id)]
    id: u64,
}

#[test]
fn existing_serialize_protocol_and_names_are_preserved() {
    fn check(value: impl Serialize, expected: &str) {
        assert_eq!(value.serialize(Probe).unwrap_err().to_string(), expected);
    }
    check(Single::new(1), "serialize_u64");
    check(Tuple::new(1, 2), "tuple:2");
    check(Named::new(1, 2), "struct:Named");
    check(Choice::Unit, "unit_variant:Choice:Unit");
    check(Choice::Newtype(1), "newtype_variant:Choice:Newtype");
    check(Choice::Tuple(1, 2), "tuple_variant:Choice:Tuple");
    check(Choice::Named { value: 1 }, "struct_variant:Choice:Named");
    check(OnlyId { id: 1 }, "struct:OnlyId");
    check(OutputDefault { id: 1 }, "struct:OutputDefault");
    check(InputDefault { id: 1 }, "struct:Output");
}

impl<'de> serde::Deserializer<'de> for Probe {
    type Error = Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        observed("other")
    }
    fn deserialize_struct<V: serde::de::Visitor<'de>>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error> {
        observed(name)
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any
    }
}
#[test]
fn directional_container_rename_keeps_the_other_domain_name() {
    assert_eq!(
        OutputDefault::deserialize(Probe).err().unwrap().to_string(),
        "Input"
    );
    assert_eq!(
        InputDefault::deserialize(Probe).err().unwrap().to_string(),
        "InputDefault"
    );
}
