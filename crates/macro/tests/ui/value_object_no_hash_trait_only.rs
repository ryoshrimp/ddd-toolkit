use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(trait_only, no_hash)]
struct Wrapped(String);

fn main() {}
