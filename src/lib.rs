//! Domain-Driven Design building blocks for Rust: entities, value objects,
//! aggregates, repositories, and event dispatch, plus derive macros for the
//! common cases. A single-dependency facade re-exporting
//! [`ddd-toolkit-core`](https://docs.rs/ddd-toolkit-core) and, behind the
//! `derive` feature (on by default),
//! [`ddd-toolkit-macro`](https://docs.rs/ddd-toolkit-macro). See this
//! crate's README for a full example.

pub use ddd_toolkit_core::*;

#[cfg(feature = "derive")]
pub use ddd_toolkit_macro::*;