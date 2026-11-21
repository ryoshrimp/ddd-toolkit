use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_hash = true)]
struct Wrapped(String);

fn main() {}
