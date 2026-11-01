// Secret replaces the removed SecretVo derive; cleanup is unconditional.
use ddd_toolkit::Secret;
use std::{cell::Cell, rc::Rc};
use zeroize::Zeroize;

struct TrackedSecret(Rc<Cell<bool>>);

impl Zeroize for TrackedSecret {
    fn zeroize(&mut self) {
        self.0.set(true);
    }
}

#[test]
fn secret_zeroizes_on_drop() {
    let wiped = Rc::new(Cell::new(false));
    let secret = Secret::new(TrackedSecret(wiped.clone()));
    assert!(!wiped.get());
    drop(secret);
    assert!(wiped.get());
}

#[test]
fn secret_redacts_formatting_and_exposes_explicitly() {
    let secret = Secret::new("sekret".to_owned());
    assert_eq!(format!("{secret:?}"), "[REDACTED]");
    assert_eq!(format!("{secret}"), "[REDACTED]");
    assert_eq!(secret.expose_secret(|value| value.len()), 6);
    assert_eq!(secret.as_inner(), "sekret");
    assert_eq!(secret.into_inner(), "sekret");
}

#[test]
fn extracting_secret_transfers_cleanup_responsibility() {
    let wiped = Rc::new(Cell::new(false));
    let mut value = Secret::new(TrackedSecret(wiped.clone())).into_inner();
    assert!(!wiped.get());
    value.zeroize();
    assert!(wiped.get());
}
