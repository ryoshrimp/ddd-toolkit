use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(error = &'static str)]
struct MissingValidate(String);

fn main() {}
