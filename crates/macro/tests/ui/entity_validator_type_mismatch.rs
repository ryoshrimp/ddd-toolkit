use ddd_toolkit_macro::Entity;

// Entity validators receive `&Self`; other argument types are rejected.
fn check_id(_: &u64) -> Result<(), &'static str> {
    Ok(())
}

#[derive(Entity)]
#[entity(validate = check_id, error = &'static str)]
struct Plain {
    #[entity(id)]
    id: u64,
}

fn check_id_serde(_: &u64) -> Result<(), &'static str> {
    Ok(())
}

#[derive(Entity)]
#[entity(validate = check_id_serde, error = &'static str, serde)]
struct WithSerde {
    #[entity(id)]
    id: u64,
}

fn main() {}
