use ddd_toolkit::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use serde_with::{SerializeAs, serde_as};
use std::collections::BTreeMap;

#[serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StoredCredentials {
    #[serde_as(serialize_as = "ExposeSecret")]
    token: Secret<String>,
    #[serde_as(serialize_as = "Option<ExposeSecret>")]
    optional: Option<Secret<String>>,
    #[serde_as(serialize_as = "Vec<ExposeSecret>")]
    tokens: Vec<Secret<String>>,
    #[serde_as(serialize_as = "BTreeMap<_, ExposeSecret>")]
    named: BTreeMap<String, Secret<String>>,
    #[serde_as(serialize_as = "Option<Vec<ExposeSecret>>")]
    nested: Option<Vec<Secret<String>>>,
}

#[test]
fn explicit_plaintext_output_composes_and_round_trips() {
    let expected = serde_json::json!({
        "token": "one", "optional": "two", "tokens": ["three"],
        "named": {"key": "four"}, "nested": ["five", "six"]
    });
    let stored: StoredCredentials = serde_json::from_value(expected.clone()).unwrap();
    let output = serde_json::to_value(&stored).unwrap();
    assert_eq!(output, expected);
    assert_eq!(
        serde_json::from_value::<StoredCredentials>(output).unwrap(),
        stored
    );
    assert_eq!(format!("{:?}", stored.token), "[REDACTED]");
}

#[test]
fn absent_and_empty_containers_round_trip() {
    for optional in [serde_json::Value::Null, serde_json::json!("two")] {
        for nested in [serde_json::Value::Null, serde_json::json!([])] {
            let expected = serde_json::json!({
                "token": "", "optional": optional, "tokens": [], "named": {}, "nested": nested
            });
            let stored: StoredCredentials = serde_json::from_value(expected.clone()).unwrap();
            assert_eq!(serde_json::to_value(&stored).unwrap(), expected);
        }
    }
}

#[test]
fn adapter_also_serializes_a_standalone_secret() {
    let token = Secret::new(String::from("plaintext"));
    let mut output = Vec::new();
    ExposeSecret::serialize_as(&token, &mut serde_json::Serializer::new(&mut output)).unwrap();
    assert_eq!(output, br#""plaintext""#);
    let restored: Secret<String> = serde_json::from_slice(&output).unwrap();
    assert_eq!(restored, token);
}
