#![cfg(feature = "secret")]

use std::sync::{Arc, Mutex};

use ddd_toolkit_core::secret::{Secret, SecretMetadata};
use zeroize::Zeroize;

#[derive(Clone)]
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
    assert_eq!(secret.expose_secret_ref(), "top-secret");
    assert_eq!(secret.expose_secret(|value| value.len()), 10);
    assert_eq!(secret.into_exposed_secret(), "top-secret");
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
fn into_exposed_secret_neither_zeroizes_nor_drops_the_returned_value() {
    let operations = Arc::new(Mutex::new(Vec::new()));
    let inner = Secret::new(Probe {
        value: b"secret".to_vec(),
        operations: Arc::clone(&operations),
    })
    .into_exposed_secret();

    assert!(operations.lock().unwrap().is_empty());
    assert_eq!(inner.value, b"secret");
    drop(inner);
    assert_eq!(*operations.lock().unwrap(), ["drop"]);
}

#[test]
fn secret_clone_secret_copies_inner() {
    let left = Secret::new(String::from("same"));
    let right = left.clone_secret();
    assert!(left.ct_eq(&right));
    assert_eq!(right.expose_secret_ref(), "same");
}

#[test]
fn clone_secret_copies_are_zeroized_independently_on_drop() {
    let operations = Arc::new(Mutex::new(Vec::new()));
    let original = Secret::new(Probe {
        value: b"secret".to_vec(),
        operations: Arc::clone(&operations),
    });
    let copy = original.clone_secret();

    drop(copy);
    assert_eq!(*operations.lock().unwrap(), ["zeroize", "drop"]);
    drop(original);
    assert_eq!(
        *operations.lock().unwrap(),
        ["zeroize", "drop", "zeroize", "drop"]
    );
}

#[test]
fn secret_implements_zeroize_on_drop() {
    fn assert_zod<T: zeroize::ZeroizeOnDrop>() {}
    assert_zod::<Secret<String>>();
}

#[test]
fn secret_metadata_is_zero_sized() {
    assert_eq!(std::mem::size_of::<SecretMetadata>(), 0);
    assert_eq!(SecretMetadata, SecretMetadata);
}

#[test]
fn secret_ct_eq_compares_bytes() {
    assert!(Secret::new(String::from("same")).ct_eq(&Secret::new(String::from("same"))));
    assert!(!Secret::new(String::from("same")).ct_eq(&Secret::new(String::from("sane"))));
    assert!(!Secret::new(String::from("same")).ct_eq(&Secret::new(String::from("same!"))));
    assert!(Secret::new(vec![1u8, 2]).ct_eq(&Secret::new(vec![1u8, 2])));
    assert!(!Secret::new(vec![1u8, 2]).ct_eq(&Secret::new(vec![1u8])));
}
