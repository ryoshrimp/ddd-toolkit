use ddd_toolkit::{Entity, ValueObject};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn roundtrip<T: Serialize + DeserializeOwned>(value: T, expected: Value) {
    assert_eq!(serde_json::to_value(&value).unwrap(), expected);
    let decoded: T = serde_json::from_value(expected.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
}

#[test]
fn oauth_grant_types_use_snake_case_in_both_directions() {
    #[derive(ValueObject, Copy)]
    #[value_object(serde)]
    #[serde(rename_all = "snake_case")]
    enum GrantType {
        AuthorizationCode,
        ClientCredentials,
        RefreshToken,
    }
    for (grant, label) in [
        (GrantType::AuthorizationCode, "authorization_code"),
        (GrantType::ClientCredentials, "client_credentials"),
        (GrantType::RefreshToken, "refresh_token"),
    ] {
        roundtrip(grant, json!(label));
    }
}

#[test]
fn variant_and_field_names_preserve_enum_validation() {
    #[derive(ValueObject)]
    #[value_object(validate = check, error = (), serde)]
    #[serde(rename = "Payload", rename_all = "snake_case")]
    enum Payload {
        #[serde(rename = "single")]
        NewType(u32),
        TwoValues(u32, u32),
        #[serde(rename_all = "camelCase")]
        NamedFields {
            field_value: u32,
            #[serde(rename = "kind")]
            r#type: String,
        },
    }
    fn check(value: &Payload) -> Result<(), ()> {
        if matches!(value, Payload::NewType(0)) {
            Err(())
        } else {
            Ok(())
        }
    }
    roundtrip(Payload::NewType(1), json!({"single": 1}));
    roundtrip(Payload::TwoValues(1, 2), json!({"two_values": [1, 2]}));
    roundtrip(
        Payload::NamedFields {
            field_value: 3,
            r#type: "x".into(),
        },
        json!({"named_fields": {"fieldValue": 3, "kind": "x"}}),
    );
    assert!(
        serde_json::from_value::<Payload>(json!({"single": 0}))
            .unwrap_err()
            .to_string()
            .contains("value object validation failed")
    );
}

#[test]
fn named_value_objects_and_entities_share_naming_and_validation() {
    #[derive(ValueObject)]
    #[value_object(validate = check_value, error = (), serde)]
    #[serde(rename_all = "camelCase")]
    struct Pair {
        field_value: u32,
        #[serde(rename = "kind")]
        r#type: String,
    }
    fn check_value(value: &Pair) -> Result<(), ()> {
        if value.field_value == 0 {
            Err(())
        } else {
            Ok(())
        }
    }
    #[derive(Entity)]
    #[entity(validate = check_entity, error = (), serde)]
    #[serde(rename = "Account", rename_all = "camelCase")]
    struct Account<T>
    where
        T: Clone,
    {
        #[entity(id)]
        #[serde(rename = "account_id")]
        id: u32,
        display_name: T,
    }
    fn check_entity<T: Clone>(value: &Account<T>) -> Result<(), ()> {
        if value.id == 0 { Err(()) } else { Ok(()) }
    }
    roundtrip(
        Pair::try_new(1, "x".into()).unwrap(),
        json!({"fieldValue": 1, "kind": "x"}),
    );
    assert!(serde_json::from_value::<Pair>(json!({"fieldValue": 0, "kind": "x"})).is_err());
    let decoded: Account<String> =
        serde_json::from_value(json!({"account_id": 42, "displayName": "Alice"})).unwrap();
    assert_eq!(*decoded.id(), 42);
    roundtrip(decoded, json!({"account_id": 42, "displayName": "Alice"}));
    assert!(
        serde_json::from_value::<Account<String>>(json!({"account_id": 0, "displayName": "Alice"}))
            .err()
            .unwrap()
            .to_string()
            .contains("entity validation failed")
    );

    #[derive(Entity)]
    #[entity(serde)]
    #[serde(rename_all = "camelCase")]
    struct SingleField {
        #[entity(id)]
        account_id: u32,
    }
    roundtrip(SingleField { account_id: 7 }, json!({"accountId": 7}));
}

// Compare every rule against native Serde derives, including acronym and raw names.
macro_rules! naming_rule {
    ($test:ident, $rule:literal) => {
        #[test]
        fn $test() {
            #[derive(ValueObject)]
            #[value_object(serde)]
            #[serde(rename_all = $rule)]
            enum Derived {
                HTTPResponse,
                NewType(u32),
                NamedFields { field_value: u32, r#type: u32 },
            }
            #[derive(serde::Serialize)]
            #[serde(rename_all = $rule)]
            enum Native {
                HTTPResponse,
                NewType(u32),
                NamedFields { field_value: u32, r#type: u32 },
            }
            roundtrip(
                Derived::HTTPResponse,
                serde_json::to_value(Native::HTTPResponse).unwrap(),
            );
            roundtrip(
                Derived::NewType(1),
                serde_json::to_value(Native::NewType(1)).unwrap(),
            );
            roundtrip(
                Derived::NamedFields {
                    field_value: 1,
                    r#type: 2,
                },
                serde_json::to_value(Native::NamedFields {
                    field_value: 1,
                    r#type: 2,
                })
                .unwrap(),
            );

            #[derive(Entity)]
            #[entity(serde)]
            #[serde(rename_all = $rule)]
            struct DerivedFields {
                #[entity(id)]
                field_id: u32,
                r#type: u32,
            }
            #[derive(serde::Serialize)]
            #[serde(rename_all = $rule)]
            struct NativeFields {
                field_id: u32,
                r#type: u32,
            }
            roundtrip(
                DerivedFields {
                    field_id: 1,
                    r#type: 2,
                },
                serde_json::to_value(NativeFields {
                    field_id: 1,
                    r#type: 2,
                })
                .unwrap(),
            );
        }
    };
}

naming_rule!(lowercase, "lowercase");
naming_rule!(uppercase, "UPPERCASE");
naming_rule!(pascal_case, "PascalCase");
naming_rule!(camel_case, "camelCase");
naming_rule!(snake_case, "snake_case");
naming_rule!(screaming_snake_case, "SCREAMING_SNAKE_CASE");
naming_rule!(kebab_case, "kebab-case");
naming_rule!(screaming_kebab_case, "SCREAMING-KEBAB-CASE");

#[test]
fn independent_serde_derives_remain_supported() {
    #[derive(ValueObject, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Independent {
        AuthorizationCode,
    }
    roundtrip(
        Independent::AuthorizationCode,
        json!({"kind": "authorization_code"}),
    );

    #[derive(Entity, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct IndependentEntity {
        #[entity(id)]
        id: u32,
    }
    roundtrip(IndependentEntity { id: 7 }, json!({"id": 7}));
}
