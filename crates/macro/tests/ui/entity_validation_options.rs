use ddd_toolkit_macro::Entity;

#[derive(Entity)]
#[entity(validate = check)]
struct MissingError {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(error = &'static str)]
struct MissingValidator {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(trait_only, serde)]
struct TraitOnlySerde {
    #[entity(id)]
    id: u64,
}

#[derive(Entity)]
#[entity(validate = check, error = &'static str)]
struct PublicState {
    #[entity(id)]
    id: u64,
    /// **English:**<br>
    /// Exposes state to test rejection of public validated fields.
    ///
    /// **日本語:**<br>
    /// 検証対象の公開フィールドの拒否を確認するために状態を公開します。
    pub state: u32,
}

#[derive(Entity)]
#[entity(serde, serde)]
struct DuplicateSerde {
    #[entity(id)]
    id: u64,
}

fn main() {}
