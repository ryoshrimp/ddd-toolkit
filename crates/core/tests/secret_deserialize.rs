#![cfg(feature = "serde")]

use ddd_toolkit_core::Secret;
use serde::{Deserialize, Deserializer};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroize;

#[derive(Deserialize)]
struct Credentials {
    token: Secret<String>,
    tokens: Option<Vec<Secret<String>>>,
}

#[test]
fn derives_deserialize_with_nested_containers() {
    let credentials: Credentials =
        serde_json::from_str(r#"{"token":"one","tokens":["two","three"]}"#).unwrap();
    assert_eq!(credentials.token.as_inner(), "one");
    assert_eq!(credentials.tokens.unwrap()[1].as_inner(), "three");
    let credentials: Credentials = serde_json::from_str(r#"{"token":"one"}"#).unwrap();
    assert!(credentials.tokens.is_none());
}

#[derive(Debug)]
struct Reject;
impl Zeroize for Reject {
    fn zeroize(&mut self) {}
}
impl<'de> Deserialize<'de> for Reject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let input = String::deserialize(deserializer)?;
        Err(serde::de::Error::custom(format!("inner rejected {input}")))
    }
}

#[test]
fn redacts_inner_errors_including_input() {
    let error = serde_json::from_str::<Secret<Reject>>(r#""private-token""#).unwrap_err();
    let message = error.to_string();
    assert!(
        message.starts_with("failed to deserialize secret"),
        "{message}"
    );
    assert!(!message.contains("private-token"));
    assert!(!message.contains("inner rejected"));
    let error = serde_json::from_str::<Secret<u32>>(r#""private-token""#).unwrap_err();
    assert!(!error.to_string().contains("private-token"));
}

#[test]
fn deserialized_value_is_zeroized_before_drop() {
    struct Tracked {
        value: String,
        zeroized: Arc<AtomicBool>,
    }
    impl Zeroize for Tracked {
        fn zeroize(&mut self) {
            self.value.zeroize();
            self.zeroized.store(true, Ordering::SeqCst);
        }
    }
    impl Drop for Tracked {
        fn drop(&mut self) {
            assert!(self.zeroized.load(Ordering::SeqCst));
            assert!(self.value.is_empty());
        }
    }
    impl<'de> Deserialize<'de> for Tracked {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Self {
                value: String::deserialize(deserializer)?,
                zeroized: Arc::new(AtomicBool::new(false)),
            })
        }
    }
    let secret: Secret<Tracked> = serde_json::from_str(r#""private-token""#).unwrap();
    let zeroized = secret.as_inner().zeroized.clone();
    assert!(!zeroized.load(Ordering::SeqCst));
    drop(secret);
    assert!(zeroized.load(Ordering::SeqCst));
}
