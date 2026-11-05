//! Regression coverage for delegated Serde wire formats and domain invariants.
#![allow(dead_code)]
use ddd_toolkit::{Entity, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(ValueObject, Copy)]
#[value_object(serde)]
#[serde(rename_all = "snake_case")]
enum GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
}

#[test]
fn oauth_grant_types_round_trip() {
    for (grant, expected) in [
        (GrantType::AuthorizationCode, "\"authorization_code\""),
        (GrantType::ClientCredentials, "\"client_credentials\""),
        (GrantType::RefreshToken, "\"refresh_token\""),
    ] {
        assert_eq!(serde_json::to_string(&grant).unwrap(), expected);
        assert_eq!(serde_json::from_str::<GrantType>(expected).unwrap(), grant);
    }
}

macro_rules! naming_case {
    ($test:ident, $rule:literal) => {
        #[test]
        fn $test() {
            #[derive(ValueObject)]
            #[value_object(serde)]
            #[serde(rename_all = $rule)]
            enum Domain {
                HTTPServer,
                OAuthToken,
                TwoWords,
            }
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = $rule)]
            enum Wire {
                HTTPServer,
                OAuthToken,
                TwoWords,
            }
            for (domain, wire) in [
                (Domain::HTTPServer, Wire::HTTPServer),
                (Domain::OAuthToken, Wire::OAuthToken),
                (Domain::TwoWords, Wire::TwoWords),
            ] {
                let expected = serde_json::to_string(&wire).unwrap();
                assert_eq!(serde_json::to_string(&domain).unwrap(), expected);
                assert_eq!(serde_json::from_str::<Domain>(&expected).unwrap(), domain);
            }
        }
    };
}
naming_case!(lower, "lowercase");
naming_case!(upper, "UPPERCASE");
naming_case!(pascal, "PascalCase");
naming_case!(camel, "camelCase");
naming_case!(snake, "snake_case");
naming_case!(screaming_snake, "SCREAMING_SNAKE_CASE");
naming_case!(kebab, "kebab-case");
naming_case!(screaming_kebab, "SCREAMING-KEBAB-CASE");

#[derive(ValueObject)]
#[value_object(validate = validate_profile, error = &'static str, serde)]
#[serde(
    rename_all(serialize = "camelCase", deserialize = "snake_case"),
    deny_unknown_fields
)]
struct Profile {
    #[serde(
        rename(serialize = "name", deserialize = "full_name"),
        alias = "old_name"
    )]
    display_name: String,
    #[serde(default)]
    login_count: u64,
    #[serde(skip)]
    cache: String,
}
fn validate_profile(value: &Profile) -> Result<(), &'static str> {
    if value.display_name.is_empty() {
        Err("empty")
    } else {
        Ok(())
    }
}

#[test]
fn field_alias_default_skip_and_directional_names_still_validate() {
    let profile: Profile = serde_json::from_str(r#"{"old_name":"Ada"}"#).unwrap();
    assert_eq!(
        serde_json::to_string(&profile).unwrap(),
        r#"{"name":"Ada","loginCount":0}"#
    );
    assert!(profile.cache().is_empty());
    for json in [r#"{"full_name":""}"#, r#"{"full_name":"Ada","unknown":1}"#] {
        assert!(serde_json::from_str::<Profile>(json).is_err());
    }
}

#[derive(ValueObject)]
#[value_object(validate = validate_message, error = &'static str, serde)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
enum Message {
    #[serde(alias = "legacy")]
    TextPayload {
        #[serde(rename = "text")]
        body: String,
    },
    Number(u64),
    Pair(u64, u64),
    Nothing,
}
fn validate_message(value: &Message) -> Result<(), &'static str> {
    if matches!(value, Message::Number(0)) {
        Err("zero")
    } else {
        Ok(())
    }
}

#[test]
fn tagged_enum_fields_and_aliases_use_serde_and_validate() {
    for (value, json) in [
        (
            Message::TextPayload {
                body: "hello".into(),
            },
            r#"{"kind":"text_payload","data":{"text":"hello"}}"#,
        ),
        (Message::Number(7), r#"{"kind":"number","data":7}"#),
        (Message::Pair(1, 2), r#"{"kind":"pair","data":[1,2]}"#),
        (Message::Nothing, r#"{"kind":"nothing"}"#),
    ] {
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
        assert_eq!(serde_json::from_str::<Message>(json).unwrap(), value);
    }
    assert_eq!(
        serde_json::from_str::<Message>(r#"{"kind":"legacy","data":{"text":"hello"}}"#).unwrap(),
        Message::TextPayload {
            body: "hello".into()
        }
    );
    assert!(
        serde_json::from_str::<Message>(r#"{"kind":"number","data":0}"#)
            .unwrap_err()
            .to_string()
            .contains("value object validation failed")
    );
}

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(untagged)]
enum Untagged {
    Text(String),
    Number(u64),
}

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
enum Internal {
    Named { field_name: u64 },
}

#[test]
fn alternative_enum_representations() {
    assert_eq!(serde_json::to_string(&Untagged::Number(3)).unwrap(), "3");
    assert_eq!(
        serde_json::from_str::<Untagged>("3").unwrap(),
        Untagged::Number(3)
    );
    let value = Internal::Named { field_name: 1 };
    let json = r#"{"type":"Named","fieldName":1}"#;
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
    assert_eq!(serde_json::from_str::<Internal>(json).unwrap(), value);
}

#[derive(Debug, Serialize, Deserialize)]
struct NonClone {
    value: u64,
}

#[derive(ValueObject)]
#[value_object(serde)]
struct GenericValue<T>(T);

#[test]
fn value_object_serialization_does_not_require_clone() {
    let value = GenericValue::new(NonClone { value: 7 });
    let json = r#"{"value":7}"#;
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
    let restored: GenericValue<NonClone> = serde_json::from_str(json).unwrap();
    assert_eq!(restored.into_inner().value, 7);
}

#[derive(Debug, Entity)]
#[entity(serde, validate = validate_record, error = &'static str)]
#[serde(rename_all = "camelCase")]
struct Record<T> {
    #[entity(id)]
    #[serde(rename = "record_id")]
    id: u64,
    #[serde(flatten)]
    payload: T,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    optional_name: Option<String>,
}
fn validate_record<T>(record: &Record<T>) -> Result<(), &'static str> {
    if record.id == 0 { Err("zero") } else { Ok(()) }
}

#[test]
fn entity_flatten_generic_non_clone_payload_and_id_preservation() {
    let record = Record::try_new(42, NonClone { value: 7 }, None).unwrap();
    let json = serde_json::to_string(&record).unwrap();
    assert_eq!(json, r#"{"record_id":42,"value":7}"#);
    let restored: Record<NonClone> = serde_json::from_str(&json).unwrap();
    assert_eq!(*restored.id(), 42);
    assert_eq!(restored.payload.value, 7);
    assert!(
        serde_json::from_str::<Record<NonClone>>(r#"{"record_id":0,"value":7}"#)
            .unwrap_err()
            .to_string()
            .contains("entity validation failed")
    );
}

mod decimal {
    use serde::{Deserialize, Deserializer, Serializer};
    pub(super) fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
#[derive(ValueObject)]
#[value_object(serde)]
struct Decimal(#[serde(with = "decimal")] u64);

#[derive(Entity)]
#[entity(serde)]
struct Custom {
    #[entity(id)]
    #[serde(
        serialize_with = "decimal::serialize",
        deserialize_with = "decimal::deserialize"
    )]
    id: u64,
}

#[test]
fn custom_codecs_receive_original_field_types_not_owned_clones() {
    assert_eq!(serde_json::to_string(&Decimal::new(7)).unwrap(), "\"7\"");
    assert_eq!(
        serde_json::from_str::<Decimal>("\"7\"").unwrap(),
        Decimal::new(7)
    );
    let value: Custom = serde_json::from_str(r#"{"id":"42"}"#).unwrap();
    assert_eq!(*value.id(), 42);
    assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"id":"42"}"#);
}

#[derive(ValueObject)]
#[value_object(serde)]
struct Borrowed<'a>(#[serde(borrow)] &'a str);

#[derive(Entity)]
#[entity(serde)]
struct GenericBorrowed<'a, '__ddd_ser, '__ddd_de, T, const N: usize>
where
    T: Eq,
{
    #[entity(id)]
    id: u64,
    #[serde(borrow)]
    text: &'a str,
    #[serde(borrow)]
    other: &'__ddd_ser str,
    #[serde(borrow)]
    third: &'__ddd_de str,
    payload: T,
    #[serde(skip)]
    marker: std::marker::PhantomData<[(); N]>,
}

#[test]
fn borrowed_lifetimes_and_const_generics() {
    let input = String::from("\"hello\"");
    let borrowed: Borrowed<'_> = serde_json::from_str(&input).unwrap();
    assert_eq!(serde_json::to_string(&borrowed).unwrap(), input);
    let input = String::from(r#"{"id":1,"text":"a","other":"b","third":"c","payload":7}"#);
    let borrowed: GenericBorrowed<'_, '_, '_, u64, 4> = serde_json::from_str(&input).unwrap();
    assert_eq!(serde_json::to_string(&borrowed).unwrap(), input);
}

#[derive(Entity)]
#[entity(serde)]
struct Skipped<T> {
    #[entity(id)]
    id: u64,
    #[serde(skip)]
    payload: T,
}
#[derive(Default)]
struct NotSerde;

#[test]
fn skipped_fields_do_not_add_serde_bounds() {
    let value = Skipped {
        id: 1,
        payload: NotSerde,
    };
    assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"id":1}"#);
    let restored: Skipped<NotSerde> = serde_json::from_str(r#"{"id":1}"#).unwrap();
    assert_eq!(*restored.id(), 1);
}

mod text_codec {
    use serde::{Deserialize, Deserializer, Serializer};
    pub(super) fn serialize<T: AsRef<str>, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.as_ref())
    }
    pub(super) fn deserialize<'de, T: From<String>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        String::deserialize(deserializer).map(T::from)
    }
}
#[derive(Entity)]
#[entity(serde)]
#[serde(bound(serialize = "T: AsRef<str>", deserialize = "T: From<String>"))]
struct Bounded<T> {
    #[entity(id)]
    id: u64,
    #[serde(with = "text_codec")]
    payload: T,
}
struct TextWithoutSerde(String);
impl AsRef<str> for TextWithoutSerde {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl From<String> for TextWithoutSerde {
    fn from(value: String) -> Self {
        Self(value)
    }
}
#[derive(Entity)]
#[entity(serde)]
struct GenericNames<S, D, __DddSerializer, __DddDeserializer> {
    #[entity(id)]
    id: u64,
    payload: (S, D, __DddSerializer, __DddDeserializer),
}

#[test]
fn explicit_bounds_and_generic_names() {
    let json = r#"{"id":7,"payload":"text"}"#;
    let value: Bounded<TextWithoutSerde> = serde_json::from_str(json).unwrap();
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
    let json = r#"{"id":7,"payload":[1,2,3,4]}"#;
    let value: GenericNames<u64, u64, u64, u64> = serde_json::from_str(json).unwrap();
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
}

// Users can still choose independent Serde derives without the macro opt-in.
#[derive(ValueObject, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Independent {
    TwoWords,
}
#[derive(Entity, Serialize, Deserialize)]
struct IndependentEntity {
    #[entity(id)]
    #[serde(rename = "key")]
    id: u64,
}
#[test]
fn separate_serde_derives_remain_available() {
    assert_eq!(
        serde_json::to_string(&Independent::TwoWords).unwrap(),
        "\"two_words\""
    );
    let value: IndependentEntity = serde_json::from_str(r#"{"key":9}"#).unwrap();
    assert_eq!(*value.id(), 9);
}
