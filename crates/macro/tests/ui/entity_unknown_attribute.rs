use ddd_toolkit_macro::Entity;

#[derive(Entity)]
struct UnknownEntity {
    #[entity(key)]
    id: u64,
}

fn main() {}
