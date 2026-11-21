use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_eq, trait_only)]
struct Wrapped(String);

fn main() {}
