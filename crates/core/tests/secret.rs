use std::sync::{Arc, Mutex};

use ddd_toolkit_core::{Secret, SecretMetadata};
use zeroize::Zeroize;

struct Probe {
    value: Vec<u8>,
    operations: Arc<Mutex<Vec<&'static str>>>,
}

impl Zeroize for Probe {
    fn zeroize(&mut self) {
        self.value.zeroize();
        self.operations.lock().unwrap().push("zeroize");
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        self.operations.lock().unwrap().push("drop");
    }
}

#[test]
fn secret_redacts_and_exposes_only_explicitly() {
    let secret = Secret::new(String::from("top-secret"));
    assert_eq!(format!("{:?}", secret), "[REDACTED]");
    assert_eq!(secret.to_string(), "[REDACTED]");
    assert_eq!(secret.as_inner(), "top-secret");
    assert_eq!(secret.expose_secret(|value| value.len()), 10);
    assert_eq!(secret.into_inner(), "top-secret");
}

#[test]
fn secret_zeroizes_then_drops_inner_exactly_once() {
    let operations = Arc::new(Mutex::new(Vec::new()));
    {
        let _secret = Secret::new(Probe {
            value: b"secret".to_vec(),
            operations: Arc::clone(&operations),
        });
    }
    assert_eq!(*operations.lock().unwrap(), ["zeroize", "drop"]);
}

#[test]
fn into_inner_neither_zeroizes_nor_drops_the_returned_value() {
    let operations = Arc::new(Mutex::new(Vec::new()));
    let inner = Secret::new(Probe {
        value: b"secret".to_vec(),
        operations: Arc::clone(&operations),
    })
    .into_inner();

    assert!(operations.lock().unwrap().is_empty());
    assert_eq!(inner.value, b"secret");
    drop(inner);
    assert_eq!(*operations.lock().unwrap(), ["drop"]);
}

#[test]
fn secret_clone_and_equality_delegate_to_inner() {
    let left = Secret::new(String::from("same"));
    let right = left.clone();
    assert_eq!(left, right);
    assert_eq!(right.as_inner(), "same");
}

#[test]
fn secret_metadata_is_zero_sized() {
    assert_eq!(std::mem::size_of::<SecretMetadata>(), 0);
    assert_eq!(SecretMetadata, SecretMetadata);
}
