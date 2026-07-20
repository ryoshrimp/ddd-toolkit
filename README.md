# ddd-toolkit

Domain-Driven Design building blocks for Rust: entities, value objects,
aggregates, repositories, and event dispatch, plus derive macros for the
common cases. This crate is a single-dependency facade over
[`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit-core) and
[`ddd-toolkit-macro`](https://github.com/ryoshrimp/ddd-toolkit-macro) - add
`ddd-toolkit` and you get both.

## Example

```rust
use ddd_toolkit::domain::{SecretVo, ValidationError, Wrapped};

#[derive(ddd_toolkit::ValueObject, Clone, PartialEq, Debug)]
#[vo(validate = "validate_email")]
struct Email(String);

fn validate_email(s: &str) -> Result<(), ValidationError> {
    if s.contains('@') {
        Ok(())
    } else {
        Err(ValidationError::new("Email", "missing @"))
    }
}

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct UserId(String);

#[derive(ddd_toolkit::SecretVo, Clone, PartialEq)]
struct ApiKey(String);

fn main() {
    let email = Email::try_from("a@b.com".to_string()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");

    let key = ApiKey::try_new("sekret".to_string()).unwrap();
    assert_eq!(format!("{key:?}"), "ApiKey(***)"); // never leaks the secret
}
```

## Features

| Feature  | Default | Enables |
|----------|---------|---------|
| `derive` | yes     | `#[derive(ValueObject, EntityId, SecretVo, EnumVo)]` via `ddd-toolkit-macro` |
| `chrono` | no      | `Clock` port + `SystemClock`/`FixedClock` adapters |
| `uuid`   | no      | `IdGenerator` port + `UuidV4Generator`/`UuidV7Generator` adapters |
| `zeroize`| no      | `#[vo(zeroize)]` support on `SecretVo` (also requires depending on the `zeroize` crate directly - see its docs) |

## Crates in this project

- **`ddd-toolkit`** (this crate) - the facade; depend on this for normal use.
- [`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit-core) - the
  domain/port traits and reference (in-memory) adapters, with no macro
  dependency.
- [`ddd-toolkit-macro`](https://github.com/ryoshrimp/ddd-toolkit-macro) - the
  derive macros, usable standalone alongside `ddd-toolkit-core` or through
  this facade.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
