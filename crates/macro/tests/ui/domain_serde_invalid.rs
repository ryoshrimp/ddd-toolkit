use ddd_toolkit_macro::{Entity, ValueObject};

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(rename_all = "not_a_serde_rule")]
enum InvalidRule {
    TwoWords,
}

#[derive(Entity)]
#[entity(serde)]
struct InvalidRename {
    #[entity(id)]
    #[serde(rename = 3)]
    id: u64,
}

fn main() {}
