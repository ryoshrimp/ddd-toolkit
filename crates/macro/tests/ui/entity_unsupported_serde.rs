use ddd_toolkit_macro::Entity;

#[derive(Entity)]
#[entity(serde)]
struct Account {
    #[entity(id)]
    id: u32,
    #[serde(skip)]
    name: String,
}

fn main() {}
