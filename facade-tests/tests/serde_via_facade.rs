//! Permanent regression test for `ddd-toolkit`'s `serde` feature: the
//! generated `Serialize`/`Deserialize` impls reference `::serde` by a bare
//! path, which only resolves if this crate depends on `serde` directly (see
//! `[dev-dependencies]` in Cargo.toml), and requires `ddd-toolkit`'s own
//! `serde` feature to be enabled.

use ddd_toolkit::domain::{EnumVo as _, ValidationError};

#[derive(ddd_toolkit::ValueObject, Clone, PartialEq, Debug)]
#[vo(validate = "validate_email")]
struct Email(String);

fn validate_email(s: &str) -> Result<(), ValidationError> {
    if s.contains('@') {
        Ok(())
    } else {
        Err(ValidationError::new("Email", "missing @"))
    }
}

#[test]
fn value_object_round_trips_through_json_via_facade() {
    let email = Email::try_from("a@b.com".to_string()).expect("valid email");
    let json = serde_json::to_string(&email).unwrap();
    assert_eq!(json, "\"a@b.com\"");

    let back: Email = serde_json::from_str(&json).unwrap();
    assert_eq!(email, back);
}

#[test]
fn value_object_deserialize_reruns_validation_via_facade() {
    let err = serde_json::from_str::<Email>("\"not-an-email\"").unwrap_err();
    assert!(err.to_string().contains("missing @"));
}

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct UserId(String);

#[test]
fn entity_id_round_trips_through_json_via_facade() {
    let id = UserId::try_from("u-1".to_string()).expect("valid id");
    let json = serde_json::to_string(&id).unwrap();
    let back: UserId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[derive(ddd_toolkit::EnumVo, Debug, Clone, Copy, PartialEq)]
#[vo(rename_all = "snake_case")]
enum Status {
    Active,
    Closed,
}

#[test]
fn enum_vo_round_trips_through_json_via_facade() {
    for status in Status::variants() {
        let json = serde_json::to_string(status).unwrap();
        let back: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(*status, back);
    }
}

#[test]
fn enum_vo_serializes_renamed_form_via_facade() {
    assert_eq!(
        serde_json::to_string(&Status::Closed).unwrap(),
        "\"closed\""
    );
}
