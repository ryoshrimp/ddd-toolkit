use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_eq, no_eq)]
struct Wrapped(String);

fn main() {}
