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
    Codes(Vec<u32>),
    Pair(u32, bool),
    EmptyTuple(),
    EmptyNamed {},
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

#[test]
fn enum_json_shapes_follow_serde_and_round_trip() {
    for (value, expected) in [
        (Status::new_pending(), r#""Pending""#),
        (Status::new_ready(3), r#"{"Ready":{"count":3}}"#),
        (Status::new_code(7), r#"{"Code":7}"#),
        (Status::new_codes(vec![7, 8]), r#"{"Codes":[7,8]}"#),
        (Status::new_codes(vec![]), r#"{"Codes":[]}"#),
        (Status::new_pair(7, true), r#"{"Pair":[7,true]}"#),
        (Status::new_emptytuple(), r#"{"EmptyTuple":[]}"#),
        (Status::new_emptynamed(), r#"{"EmptyNamed":{}}"#),
    ] {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, expected);
        assert_eq!(serde_json::from_str::<Status>(&json).unwrap(), value);
    }
}

#[test]
fn legacy_tuple_wrapper_is_not_accepted_as_a_newtype_value() {
    // Migration must remove exactly the old outer tuple layer. Automatically
    // unwrapping sequences would misinterpret valid container-valued newtypes.
    assert!(serde_json::from_str::<Status>(r#"{"Code":[7]}"#).is_err());
    assert!(serde_json::from_str::<Status>(r#"{"Codes":[[7,8]]}"#).is_err());
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = validate_code, error = &'static str, serde)]
enum ValidatedStatus {
    Code(u32),
}

fn validate_code(value: &ValidatedStatus) -> Result<(), &'static str> {
    match value {
        ValidatedStatus::Code(0) => Err("code must be positive"),
        _ => Ok(()),
    }
}

#[test]
fn newtype_enum_deserialization_still_validates() {
    let value = ValidatedStatus::try_new_code(7).unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Code":7}"#);
    assert_eq!(
        serde_json::from_str::<ValidatedStatus>(&json).unwrap(),
        value
    );
    assert_eq!(
        ValidatedStatus::try_new_code(0),
        Err("code must be positive")
    );
    let error = serde_json::from_str::<ValidatedStatus>(r#"{"Code":0}"#).unwrap_err();
    assert!(error.to_string().contains("value object validation failed"));
}
