use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(tag = "kind")]
enum GrantType {
    AuthorizationCode,
}

fn main() {}
