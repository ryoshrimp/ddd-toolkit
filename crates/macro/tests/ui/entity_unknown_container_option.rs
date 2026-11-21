use ddd_toolkit_macro::Entity;

#[derive(Entity)]
#[entity(getter)]
struct User {
    #[entity(id)]
    id: u32,
}

fn main() {}
