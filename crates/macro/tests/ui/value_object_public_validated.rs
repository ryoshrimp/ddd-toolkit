use ddd_toolkit_macro::ValueObject;

fn validate(value: &String) -> Result<(), &'static str> {
    if value.is_empty() { Err("empty") } else { Ok(()) }
}

/// **English:** Exposes a validated field to check that unrestricted public access is rejected.
///
/// **日本語:** 検証付きフィールドを公開し、無制限の公開が拒否されることを確認します。
#[derive(ValueObject)]
#[value_object(validate = validate, error = &'static str)]
pub struct PublicValue(pub String);

fn main() {}
