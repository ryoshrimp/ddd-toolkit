use ddd_toolkit_macro::ValueObject;

fn validate(_: &Pct) -> Result<(), &'static str> {
    Ok(())
}

/// **English:** Validates an enum to check that variants cannot be protected.
///
/// **日本語:** 列挙型に検証を指定し、バリアントを保護できないため拒否されることを確認します。
#[derive(ValueObject)]
#[value_object(validate = validate, error = &'static str)]
enum Pct {
    Value(u8),
}

fn main() {}
