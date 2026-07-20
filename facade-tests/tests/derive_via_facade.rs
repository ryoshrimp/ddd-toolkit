//! Every test in this crate uses only `ddd_toolkit::` paths - this package's
//! `Cargo.toml` depends on nothing but the `ddd-toolkit` facade, simulating
//! a real downstream consumer. It exists specifically to catch the class of
//! bug where a derive macro's generated code only resolves when the caller
//! also depends on `ddd-toolkit-core` directly.

use ddd_toolkit::domain::{EnumVo as _, SecretVo as _, ValidationError, Wrapped as _};

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
fn value_object_round_trips_through_facade() {
    let email = Email::try_from("a@b.com".to_string()).expect("valid email");
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(email.to_string(), "a@b.com");
    assert_eq!(email.into_inner(), "a@b.com".to_string());
}

#[test]
fn value_object_validation_runs_through_facade() {
    let err = Email::try_from("not-an-email".to_string()).unwrap_err();
    assert_eq!(err, ValidationError::new("Email", "missing @"));
}

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct UserId(String);

#[test]
fn entity_id_satisfies_bounds_through_facade() {
    fn assert_entity_id<T: ddd_toolkit::domain::EntityId>() {}
    assert_entity_id::<UserId>();

    let a = UserId("u-1".to_string());
    let b = UserId("u-1".to_string());
    let c = UserId("u-2".to_string());
    assert_eq!(a, b);
    assert!(a < c);
    assert_eq!(a.to_string(), "u-1");
}

#[derive(ddd_toolkit::SecretVo, Clone, PartialEq)]
struct ApiKey(String);

#[test]
fn secret_vo_redacts_debug_through_facade() {
    let key = ApiKey::try_new("sekret".to_string()).expect("valid secret");
    assert_eq!(key.expose_secret(), "sekret");
    assert_eq!(format!("{key:?}"), "ApiKey(***)");
}

#[derive(ddd_toolkit::EnumVo, Debug, Clone, Copy, PartialEq)]
#[vo(rename_all = "snake_case")]
enum Status {
    Active,
    Closed,
}

#[test]
fn enum_vo_round_trips_through_facade() {
    assert_eq!(Status::Active.to_string(), "active");
    assert_eq!("closed".parse::<Status>(), Ok(Status::Closed));
    assert_eq!(Status::variants().len(), 2);
}
