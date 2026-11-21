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

#[derive(Entity)]
#[entity(getters)]
struct RawConflictingId {
    #[entity(id)]
    key: u64,
    r#id: String,
}

#[derive(Entity)]
#[entity(getters, validate = check, error = ())]
struct ConflictingTryNew {
    #[entity(id)]
    id: u64,
    try_new: String,
}

fn check(_: &ConflictingTryNew) -> Result<(), ()> {
    Ok(())
}

#[derive(Entity)]
#[entity(getters, validate = check_raw, error = ())]
struct RawConflictingTryNew {
    #[entity(id)]
    id: u64,
    r#try_new: String,
}

fn check_raw(_: &RawConflictingTryNew) -> Result<(), ()> {
    Ok(())
}

fn main() {}
