use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_hash, no_hash)]
struct Wrapped(String);

fn main() {}
