// Macro 0.4.0 emits `field: field` in named ValueObject constructors.
// Clippy 1.99 checks those external macro spans; keep coverage until upstream
// generates shorthand fields. This allowance is limited to this test crate.
#![allow(clippy::redundant_field_names)]

// Serde generation is opt-in per value object and uses core's serde re-export.
// This consumer intentionally has no direct serde dependency.
#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = validate_email, error = &'static str, serde)]
struct Email(String);

fn validate_email(value: &str) -> Result<(), &'static str> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("missing @")
    }
}

#[test]
fn validated_value_object_round_trips_through_json() {
    let email = Email::try_new("a@b.com".to_owned()).unwrap();
    let json = serde_json::to_string(&email).unwrap();
    assert_eq!(json, "\"a@b.com\"");
    assert_eq!(serde_json::from_str::<Email>(&json).unwrap(), email);
}

#[test]
fn deserialization_runs_validation() {
    let error = serde_json::from_str::<Email>("\"invalid\"").unwrap_err();
    assert!(error.to_string().contains("value object validation failed"));
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(serde)]
struct Address {
    city: String,
    postal_code: u32,
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(serde)]
enum Status {
    Pending,
    Ready { count: u32 },
    Code(u32),
}

#[test]
fn named_value_object_and_externally_tagged_enum_round_trip() {
    let address = Address::new("Tokyo".to_owned(), 100);
    let json = serde_json::to_string(&address).unwrap();
    assert_eq!(serde_json::from_str::<Address>(&json).unwrap(), address);
    assert_eq!(
        serde_json::to_string(&Status::new_pending()).unwrap(),
        "\"Pending\""
    );
    let ready = Status::new_ready(3);
    let json = serde_json::to_string(&ready).unwrap();
    assert_eq!(json, "{\"Ready\":{\"count\":3}}");
    assert_eq!(serde_json::from_str::<Status>(&json).unwrap(), ready);
}

// Upstream 0.4.0 serializes a single tuple field as a sequence, but its
// generated serde deserializer expects a newtype variant instead.
#[test]
fn upstream_single_field_enum_json_limitation_is_visible() {
    let json = serde_json::to_string(&Status::new_code(7)).unwrap();
    assert_eq!(json, "{\"Code\":[7]}");
    let error = serde_json::from_str::<Status>(&json).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("invalid type: sequence, expected u32")
    );
    assert_eq!(
        serde_json::from_str::<Status>("{\"Code\":7}").unwrap(),
        Status::new_code(7)
    );
}
