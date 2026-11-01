use ddd_toolkit_macro::Entity;

struct NotHash;

#[derive(Entity)]
struct Item {
    #[entity(id)]
    id: NotHash,
}

fn main() {
    let item = Item { id: NotHash };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&item, &mut hasher);
}
