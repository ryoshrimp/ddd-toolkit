use ddd_toolkit_macro::Entity;

#[derive(Entity)]
#[entity(getters, getters)]
struct Duplicate {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(getters = true)]
struct Valued {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(trait_only, getters)]
struct TraitOnly {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
struct SkipWithoutGetters {
    #[entity(id)]
    id: u64,
    #[entity(skip_getter)]
    secret: String,
}

#[derive(Entity)]
#[entity(getters)]
struct DuplicateSkip {
    #[entity(id)]
    id: u64,
    #[entity(skip_getter, skip_getter)]
    secret: String,
}

#[derive(Entity)]
#[entity(getters)]
struct ValuedSkip {
    #[entity(id)]
    id: u64,
    #[entity(skip_getter = true)]
    secret: String,
}

#[derive(Entity)]
#[entity(skip_getter)]
struct MisplacedSkip {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
struct MisplacedGetters {
    #[entity(id)]
    id: u64,
    #[entity(getters)]
    secret: String,
}

#[derive(Entity)]
#[entity(getters)]
struct ConflictingId {
    #[entity(id)]
    key: u64,
    id: String,
}

fn main() {}
