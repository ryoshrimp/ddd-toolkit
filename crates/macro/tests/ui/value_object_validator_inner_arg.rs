use ddd_toolkit_macro::ValueObject;

// Validators receive `&Self`; the former `&Inner` form is rejected.
fn validate_tuple(value: &String) -> Result<(), &'static str> {
    if value.is_empty() { Err("empty") } else { Ok(()) }
}

fn validate_named(value: &String) -> Result<(), &'static str> {
    if value.is_empty() { Err("empty") } else { Ok(()) }
}

#[derive(ValueObject)]
#[value_object(validate = validate_tuple, error = &'static str)]
struct TupleEmail(String);

#[derive(ValueObject)]
#[value_object(validate = validate_named, error = &'static str, serde)]
struct NamedEmail {
    value: String,
}

fn main() {}
