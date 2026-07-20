//! Domain-Driven Design building blocks for Rust: entities, value objects,
//! aggregates, repositories, and event dispatch, plus derive macros for the
//! common cases. A single-dependency facade re-exporting
//! [`ddd-toolkit-core`](https://docs.rs/ddd-toolkit-core) and, behind the
//! `derive` feature (on by default),
//! [`ddd-toolkit-macro`](https://docs.rs/ddd-toolkit-macro). See this
//! crate's README for a full example.
//!
//! # Example
//!
//! ```
//! use ddd_toolkit::domain::{SecretVo, ValidationError, Wrapped};
//!
//! #[derive(ddd_toolkit::ValueObject, Clone, PartialEq, Debug)]
//! #[vo(validate = "validate_email")]
//! struct Email(String);
//!
//! fn validate_email(s: &str) -> Result<(), ValidationError> {
//!     if s.contains('@') {
//!         Ok(())
//!     } else {
//!         Err(ValidationError::new("Email", "missing @"))
//!     }
//! }
//!
//! #[derive(ddd_toolkit::SecretVo, Clone, PartialEq)]
//! struct ApiKey(String);
//!
//! let email = Email::try_from("a@b.com".to_string()).unwrap();
//! assert_eq!(email.as_inner(), "a@b.com");
//!
//! let key = ApiKey::try_new("sekret".to_string()).unwrap();
//! assert_eq!(format!("{key:?}"), "ApiKey(***)"); // never leaks the secret
//! ```

pub use ddd_toolkit_core::*;

#[cfg(feature = "derive")]
pub use ddd_toolkit_macro::*;
