use ddd_toolkit_macro::Entity;

#[derive(Entity)]
struct User {
    #[entity(id)]
    id: u64,
    name: String,
}

fn main() {
    let user = User {
        id: 1,
        name: String::from("user"),
    };
    assert_eq!(user.id(), &1);
}
