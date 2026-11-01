use ddd_toolkit_macro::ValueObject;

fn validate(_: &String) -> Result<(), &'static str> {
    Ok(())
}

#[derive(ValueObject)]
#[value_object(validate = validate)]
struct MissingError(String);

fn main() {}
