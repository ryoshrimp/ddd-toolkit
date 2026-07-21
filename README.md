# ddd-toolkit

[![crates.io](https://img.shields.io/crates/v/ddd-toolkit.svg)](https://crates.io/crates/ddd-toolkit)
[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit)](https://docs.rs/ddd-toolkit)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/ddd-toolkit)](Cargo.toml)
[![license](https://img.shields.io/crates/l/ddd-toolkit.svg)](#license)

Domain-Driven Design building blocks for Rust: entities, value objects,
aggregates, repositories, and event dispatch, plus derive macros for the
common cases. This crate is a single-dependency facade over
[`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit-core) and
[`ddd-toolkit-macro`](https://github.com/ryoshrimp/ddd-toolkit-macro) - add
`ddd-toolkit` and you get both, re-exported under one path.

Reach for `ddd-toolkit-core` directly instead if you want the traits with
zero macro machinery and full control over every impl.

## Installation

```sh
cargo add ddd-toolkit
```

Or add it to `Cargo.toml` directly, enabling the features you need:

```toml
[dependencies]
ddd-toolkit = { version = "0.1", features = ["chrono", "uuid"] }
```

`derive` is on by default; disable default features if you'd rather
implement `ValueObject`/`EntityId`/`SecretVo`/`EnumVo` by hand. Requires
Rust 1.85 or newer (edition 2024).

## What's here

Everything is reachable directly under `ddd_toolkit::`, whichever of the two
underlying crates it originates in:

- **`domain`** - `Entity`, `AggregateRoot`, `ValueObject`/`Wrapped`,
  `EntityId`, `SecretVo`, `EnumVo`, `DomainEvent`, `ValidationError`. With
  `derive` enabled, `#[derive(ValueObject, EntityId, SecretVo, EnumVo)]`
  generates the boilerplate for those four.
- **`port`** - `Load`/`Save`/`Delete` repository traits, `EventDispatcher`
  (with `DispatchError<E>` reporting undelivered events on partial
  failure), `Clock` (behind `chrono`), `IdGenerator`.
- **`adapter`** - real port implementations, gated per feature so you only
  pull in what you use: `SystemClock` behind `chrono`,
  `UuidV4Generator`/`UuidV7Generator` behind `uuid`.
- **`mock`** - test doubles: `InMemoryStore` (a `Load`/`Save`/`Delete`
  implementation backed by a `HashMap`) and `FixedIdGenerator` build
  unconditionally; `FixedClock` is behind the `chrono` feature.
- **`application`** - `UseCase`.

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

`EntityId` generates the same `Wrapped`/`TryFrom`/`Display` impls as
`ValueObject`, plus `Eq`/`Hash`/`Ord`/`PartialOrd` so the id can be used as a
map key or sorted. A real aggregate implements `AggregateRoot` to record
domain events, and is persisted through the `Load`/`Save` ports - here via
`mock::repository::InMemoryStore`, a drop-in test double:

```rust
use ddd_toolkit::domain::{AggregateRoot, DomainEvent, Entity};
use ddd_toolkit::mock::repository::InMemoryStore;
use ddd_toolkit::port::repository::{Load, Save};

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct OrderId(String);

#[derive(Debug, Clone, PartialEq)]
struct OrderPlaced;

impl DomainEvent for OrderPlaced {}

#[derive(Debug, Clone)]
struct Order {
    id: OrderId,
    events: Vec<OrderPlaced>,
}

impl Entity for Order {
    type Id = OrderId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Order {
    type Event = OrderPlaced;

    fn record(&mut self, event: Self::Event) {
        self.events.push(event);
    }

    fn take_events(&mut self) -> Vec<Self::Event> {
        std::mem::take(&mut self.events)
    }
}

#[tokio::main]
async fn main() -> Result<(), ddd_toolkit::port::PortError> {
    let store = InMemoryStore::new();
    let id = OrderId::try_from("order-1".to_string()).unwrap();
    let mut order = Order { id: id.clone(), events: vec![OrderPlaced] };

    store.save(&mut order).await?; // drains the aggregate's recorded events as a side effect
    assert!(order.take_events().is_empty());

    let loaded = store.load(&id).await?.expect("just-saved order should be found");
    assert_eq!(loaded.id, id);
    Ok(())
}
```

More usage, including `EventDispatcher` and the `chrono`/`uuid` adapters, is
covered in the [API docs](https://docs.rs/ddd-toolkit).

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

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
