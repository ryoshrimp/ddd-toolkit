# ddd-toolkit

English | [日本語](README.ja.md)

[![crates.io](https://img.shields.io/crates/v/ddd-toolkit.svg)](https://crates.io/crates/ddd-toolkit)
[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit)](https://docs.rs/ddd-toolkit)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/ddd-toolkit.svg)](#license)

Domain-Driven Design traits and helpers, re-exported from
[`ddd-toolkit-core`](crates/core) 0.4.3,
plus the four derives from
[`ddd-toolkit-macro`](crates/macro) 0.5.1.
All public items are available at `ddd_toolkit::`.

Tested with the latest stable Rust (edition 2024). No minimum supported Rust
version (MSRV) is specified.

This repository is a Cargo workspace containing the facade, core, and macro
crates. They remain separate crates.io packages, developed and tested together.
See [CONTRIBUTING.md](CONTRIBUTING.md) and [release instructions](docs/releasing.md).

## Installation

For traits and helpers without derive macros:

```toml
[dependencies]
ddd-toolkit = { version = "0.4", default-features = false }
```

For derive macros (enabled by default):

```toml
[dependencies]
ddd-toolkit = "0.4"
```

Derives work with the facade as the only toolkit dependency. Generated paths
prefer a direct `ddd-toolkit-core` dependency when present, otherwise they use
the facade's re-exports. Both core and facade dependencies may be renamed.

Facade 0.3.0 adopts core/macro 0.4.0 and contains breaking API changes
from facade 0.2. See the migration guide below and [CHANGELOG.md](CHANGELOG.md).

## What's here

- `Entity`, `ValueObject`, `AggregateRoot`, and `DomainEvent` traits and derives.
- `Repository` / `AsyncRepository`, `DomainEventPublisher` /
  `AsyncDomainEventPublisher`, and `dispatch_events` / `dispatch_events_async`.
- CQRS command/query handlers and lifecycle contexts.
- Event stores, revision types, aggregate replay, and snapshots.
- Event consumption, projections, checkpoints, and transactional outbox contracts.
- Specifications, domain services/policies/factories, process managers,
  units of work, and event upcasters.
- `Secret<T>` for redacted formatting and zeroization on drop.

Core defines contracts, not storage adapters, clock or ID generators, an async
runtime, or automatic transaction/delivery guarantees. Implement adapters in your
application. Async ports return boxed `Send` futures without choosing a runtime.

## Example

With both dependencies shown above:

```rust
#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = validate_email, error = &'static str)]
struct Email(String);

fn validate_email(value: &str) -> Result<(), &'static str> {
    if value.contains('@') { Ok(()) } else { Err("missing @") }
}

fn main() {
    let email = Email::try_new("a@b.com".to_owned()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(Email::try_new("invalid".to_owned()), Err("missing @"));

    let key = ddd_toolkit::Secret::new("sekret".to_owned());
    assert_eq!(format!("{key:?}"), "[REDACTED]");
    assert_eq!(key.expose_secret(|value| value.len()), 6);
}
```

`ValueObject` derives `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`;
do not derive these again. Unvalidated structs use `new`; validated structs use
`try_new`. Named/multi-field structs and data-carrying enums are also supported.
`Entity` derives equality and hashing using only the `#[entity(id)]` field.

### Aggregate event publication

```rust
use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, dispatch_events};

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct OrderPlaced;

#[derive(ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<OrderPlaced>,
}

struct Publisher;

impl DomainEventPublisher for Publisher {
    type Event = OrderPlaced;
    type Error = &'static str;

    fn publish(&mut self, _event: &OrderPlaced) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let mut order = Order { id: 1, events: vec![] };
    order.record_event(OrderPlaced);
    dispatch_events(&mut order, &mut Publisher).unwrap();
    assert!(order.pull_events().is_empty());
}
```

Repository persistence and event publication are separate. `Repository::save`
borrows the aggregate immutably and does not automatically drain pending events.
`dispatch_events` drains events on success; on publication failure it restores
**all** pulled events. Already published events are not rolled back, so retrying
may deliver duplicates. Application adapters define transaction guarantees.

## Features

| Feature | Default | Enables |
|---------|---------|---------|
| `derive` | yes | `ValueObject`, `Entity`, `DomainEvent`, `AggregateRoot` derives; no direct core dependency required |
| `serde` | no | Core's serde re-export and serde derive support for `#[value_object(serde)]`; independent of `derive` |

To serialize a value object, enable the facade's `serde` feature and opt in on
the type:

```rust
#[derive(ddd_toolkit::ValueObject)]
#[value_object(serde)]
struct UserId(String);
```

No direct `serde` dependency is required for these generated implementations.
Validated deserialization runs the validator and maps failure to
`value object validation failed`. Enums use externally tagged variants.
When combining serde and validation in one attribute, place the `serde` flag
last (`#[value_object(validate = check, error = Error, serde)]`): macro 0.4.1
rejects a flag followed by other options.

Single-field tuple enum variants use Serde's newtype format: `Code(u32)` emits
`{"Code":7}` and round-trips. A container field retains its own shape, so
`Codes(Vec<u32>)` emits `{"Codes":[7,8]}`. Unit, named, and multi-field tuple
variants retain their existing formats.

### Migrating serialized enums from macro 0.4

Facade 0.4.0 / macro 0.5.0 intentionally change the output format, not in a patch
release. Older serializers emitted an extra tuple wrapper (`{"Code":[7]}`),
although their deserializers already expected the newtype format. Before loading
saved data, remove exactly one outer array layer for each affected single-field
tuple variant: `{"Code":[7]}` → `{"Code":7}`, or `{"Codes":[[7,8]]}` →
`{"Codes":[7,8]}`. Use your schema/version metadata to identify old records; do
not unwrap all arrays. There is no automatic legacy-format fallback because
container-valued fields can make the two representations ambiguous. Coordinate
producer/consumer upgrades or pin facade 0.3 / macro 0.4 until migration is ready.
The same newtype-vs-tuple change applies to other Serde formats; migrate them
using their format-specific APIs.

Macro 0.5.1 generates shorthand fields in named value-object constructors,
including validated constructors and named enum variants. No
`clippy::redundant_field_names` allowance is needed. See the
[core/macro compatibility contract](docs/macro-compatibility.md) for supported
versions and serde feature requirements.

`Secret<T>` is available without a feature flag and is not automatically serialized.
Calling `Secret::into_inner` transfers zeroization responsibility to the caller.

## Migrating from core/macro 0.3

- Replace `domain`, `port`, `adapter`, `mock`, and `application` module paths
  with root-level imports. Built-in adapters/test doubles and `UseCase` are gone.
- Replace `EntityId` / `EnumVo` derives with suitable `ValueObject` types;
  `Wrapped` is replaced by `ValueObject`'s `as_inner` / `into_inner` methods.
- Replace `#[vo(validate = "function")]` with
  `#[value_object(validate = function, error = YourError)]`.
  `ValidationError` is gone; choose your own error type.
- Replace generated `TryFrom` / `From` / `Display` usage with constructors and
  explicit application implementations where needed.
- Replace `SecretVo` / `#[vo(zeroize)]` with `Secret<T>`.
- Replace `record` / `take_events` with `record_event` / `pull_events`.
- Replace `Load` / `Save` / `Delete` with `Repository` or `AsyncRepository`,
  and `EventDispatcher` with event publisher contracts. `PortError` is gone;
  adapters choose their own error types.
- Remove `chrono`, `uuid`, and `zeroize` feature requests.
- Opt in to serde per value object; a facade-only dependency supports derives.

More details are in the [API docs](https://docs.rs/ddd-toolkit). The
[`facade-tests`](facade-tests/tests) integration suite covers custom repositories,
async ports, multi-aggregate flows, publication failure/retry, derives, serde,
and secrets through the facade.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
