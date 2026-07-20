//! Permanent regression test for the `#[vo(zeroize)]` bug fixed earlier:
//! the generated `Drop` impl requires this crate to depend on `zeroize`
//! directly (see `[dev-dependencies]` in Cargo.toml), and requires
//! `ddd-toolkit`'s own `zeroize` feature to be enabled.

use ddd_toolkit::domain::SecretVo as _;

#[derive(ddd_toolkit::SecretVo, Clone, PartialEq)]
#[vo(zeroize)]
struct WipedSecret([u8; 4]);

#[test]
fn zeroize_attribute_generates_working_drop_impl_through_facade() {
    let secret = WipedSecret::try_new([1, 2, 3, 4]).expect("valid secret");
    assert_eq!(secret.expose_secret(), &[1, 2, 3, 4]);
    drop(secret);
}
