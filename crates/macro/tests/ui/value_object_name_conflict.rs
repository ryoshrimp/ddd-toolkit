use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct NewField {
    new: u32,
    end: u32,
}

#[derive(ValueObject)]
struct AsInnerField {
    as_inner: u32,
    end: u32,
}

#[derive(ValueObject)]
struct IntoInnerField {
    into_inner: u32,
    end: u32,
}

#[derive(ValueObject)]
#[value_object(validate = check, error = ())]
struct TryNewField {
    try_new: u32,
    end: u32,
}

fn check(_: &TryNewField) -> Result<(), ()> {
    Ok(())
}

#[derive(ValueObject)]
#[value_object(validate = check_raw, error = ())]
struct RawTryNewField {
    r#try_new: u32,
    end: u32,
}

fn check_raw(_: &RawTryNewField) -> Result<(), ()> {
    Ok(())
}

#[derive(ValueObject)]
struct RawNewField {
    r#new: u32,
    end: u32,
}

#[derive(ValueObject)]
enum InnerVariant {
    Inner,
    Other,
}

#[derive(ValueObject)]
enum LowerInnerVariant {
    INNER,
}

#[derive(ValueObject)]
enum RawInnerVariant {
    r#Inner,
}

fn main() {}
