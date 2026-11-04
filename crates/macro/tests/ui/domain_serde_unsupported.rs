use ddd_toolkit_macro::{Entity, ValueObject};

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(from = "u64")]
struct FromValue(u64);

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(try_from = "u64")]
struct TryFromValue(u64);

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(into = "u64")]
struct IntoValue(u64);

#[derive(Entity)]
#[entity(serde)]
#[serde(default)]
struct DefaultEntity {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(serde)]
#[serde(crate = "serde")]
struct CrateOverride {
    #[entity(id)]
    id: u64,
}

#[derive(ValueObject)]
#[value_object(serde)]
enum VariantCodec {
    #[serde(serialize_with = "codec")]
    Value(u64),
}

#[derive(Entity)]
#[entity(serde)]
struct Getter {
    #[entity(id)]
    #[serde(getter = "get_id")]
    id: u64,
}

#[derive(ValueObject)]
#[value_object(serde)]
struct Unknown(#[serde(not_a_serde_option)] u64);

fn main() {}
