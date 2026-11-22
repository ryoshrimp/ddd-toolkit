use ddd_toolkit_macro::ValueObject;

fn validate(_: &MissingError) -> Result<(), &'static str> {
    Ok(())
}

#[derive(ValueObject)]
#[value_object(validate = validate)]
struct MissingError(String);

fn main() {}
