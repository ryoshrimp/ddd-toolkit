use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_eq = true)]
struct Wrapped(String);

fn main() {}
