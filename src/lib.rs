//! Domain-Driven Design building blocks for Rust, plus derive macros for
//! the common cases, behind a single dependency.
//!
//! This crate is a facade: it re-exports
//! [`ddd-toolkit-core`](https://docs.rs/ddd-toolkit-core) in full, plus,
//! behind the `derive` feature (on by default),
//! [`ddd-toolkit-macro`](https://docs.rs/ddd-toolkit-macro)'s derives.
//! Everything below is reachable directly under `ddd_toolkit::`, whichever
//! of the two it originates in.
//!
//! - [`domain`] - the domain-layer traits: [`domain::Entity`],
//!   [`domain::AggregateRoot`], [`domain::ValueObject`]/[`domain::Wrapped`],
//!   [`domain::EntityId`], [`domain::SecretVo`], [`domain::EnumVo`],
//!   [`domain::DomainEvent`]. With `derive` enabled,
//!   `#[derive(ValueObject, EntityId, SecretVo, EnumVo)]` generates the
//!   boilerplate for those four.
//! - [`port`] - the ports a domain depends on: repository (`Load`/`Save`/
//!   `Delete`), [`port::event::EventDispatcher`], `port::clock::Clock`
//!   (behind `chrono`), [`port::id::IdGenerator`].
//! - [`adapter`] - real adapters for those ports, behind the `chrono`/`uuid`
//!   features.
//! - [`mock`] - in-memory/fixed adapters, useful as test doubles or a
//!   starting point for a real backend.
//! - [`application`] - [`application::usecase::UseCase`].
//!
//! # Features
//!
//! | Feature   | Default | Enables |
//! |-----------|---------|---------|
//! | `derive`  | yes | `#[derive(ValueObject, EntityId, SecretVo, EnumVo)]` |
//! | `chrono`  | no  | `port::clock::Clock` and its [`adapter`]/[`mock`] implementations |
//! | `uuid`    | no  | the `uuid`-backed [`port::id::IdGenerator`] adapter |
//! | `zeroize` | no  | `#[vo(zeroize)]` support on [`domain::SecretVo`] (also needs the `zeroize` crate as a direct dependency - see its docs) |
//! | `serde`   | no  | `Serialize`/`Deserialize` on [`domain::ValueObject`]/[`domain::EntityId`]/[`domain::EnumVo`] derives, not [`domain::SecretVo`] (also needs the `serde` crate as a direct dependency - see its docs) |
//!
//! # Examples
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
//!
//! See this crate's README for a longer walkthrough, including
//! [`domain::EntityId`] and wiring an [`domain::AggregateRoot`] through a
//! [`port::repository`]/[`port::event::EventDispatcher`] pair.

pub use ddd_toolkit_core::*;

#[cfg(feature = "derive")]
pub use ddd_toolkit_macro::*;
