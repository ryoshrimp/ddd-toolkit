# ddd-toolkit

English | [日本語](README.ja.md)

[![crates.io](https://img.shields.io/crates/v/ddd-toolkit.svg)](https://crates.io/crates/ddd-toolkit)
[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit)](https://docs.rs/ddd-toolkit)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/ddd-toolkit.svg)](#license)

A Rust toolkit for developers who want to introduce Domain-Driven Design (DDD)
contracts and derives into their application: express identity, value validation,
aggregate event queues, and domain events without writing the corresponding trait
implementations by hand. It complements your existing storage and runtime choices;
it is not a framework that automatically implements persistence, transactions, or
delivery guarantees.

**Experimental, 0.x:** APIs may change. See the existing
[compatibility and breaking-change policy](docs/macro-compatibility.md#api-stability-for-generated-code).
Feedback on APIs, documentation, and use cases is welcome via
[GitHub Issues](https://github.com/ryoshrimp/ddd-toolkit/issues).

Start with the [four derives and quick start](#four-derives), continue with the
[order tutorial](examples/README.md#start-here), then explore [Advanced](#advanced)
when you need additional contracts.

## Package and toolchain

Domain-Driven Design traits and helpers, re-exported from
[`ddd-toolkit-core`](crates/core) 0.4.4,
plus the four derives from
[`ddd-toolkit-macro`](crates/macro) 0.5.1.
All public items are available at `ddd_toolkit::`.

Tested with the latest stable Rust (edition 2024). No minimum supported Rust
version (MSRV) is specified.

This repository is a Cargo workspace containing the facade, core, and macro
crates. They remain separate crates.io packages, developed and tested together.
See [CONTRIBUTING.md](CONTRIBUTING.md) and [release instructions](docs/releasing.md).

## Installation

For the quick start, use the default features (`derive` is enabled):

```toml
[dependencies]
ddd-toolkit = "0.4"
```

For traits and helpers without derive macros:

```toml
[dependencies]
ddd-toolkit = { version = "0.4", default-features = false }
```

Derives work with the facade as the only toolkit dependency. Generated paths
prefer a direct `ddd-toolkit-core` dependency when present, otherwise they use
the facade's re-exports. Both core and facade dependencies may be renamed.

Facade 0.3.0 adopts core/macro 0.4.0 and contains breaking API changes
from facade 0.2. See the migration guide below and [CHANGELOG.md](CHANGELOG.md).

## Four derives

| Derive | Start with |
| --- | --- |
| `ValueObject` | Values with equality and optional validation |
| `Entity` | Identity-based equality and hashing via `#[entity(id)]` |
| `AggregateRoot` | Pending domain events in a `#[aggregate(events)]` queue |
| `DomainEvent` | A domain event type implementing the event contract |

### Quick start

After adding the dependency with `derive` enabled as shown above, save this as
`src/main.rs` and run `cargo run`:

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
}
```

`ValueObject` derives `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`;
do not derive these again. Unvalidated structs use `new`; validated structs use
`try_new`. Named/multi-field structs and data-carrying enums are also supported.
`Entity` derives equality and hashing using only the `#[entity(id)]` field.

## Next: runnable examples

**New to DDD? Continue with the [order tutorial](examples/README.md#start-here)**:
express an order's business rules in code, then save the aggregate and send a
notification. The guide maps terminology to code and explains reading order.
From a checkout, start with:

```sh
cargo run -p ddd-toolkit --example getting_started --features derive --locked
```

Then try [CQRS](examples/cqrs.rs) for command/query handlers, Specification and
Policy (`cargo run -p ddd-toolkit --example cqrs --features derive --locked`).
Both use only `ddd_toolkit::` APIs and check results with assertions.
For retries, event sourcing, and outbox examples, see [Advanced](#advanced).

## Advanced

Use these contracts as needed; they are not prerequisites for the four derives:

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
A typical `Box::pin` implementation allocates on the heap for a non-zero-sized
future, and polling `dyn Future` uses dynamic dispatch. Type erasure supports
`dyn` port usage, and explicit lifetime/`Send` bounds express borrowing and
thread-safety requirements. These are trade-offs, not measured performance
problems. Runtime independence does not allow `!Send` futures: they do not fit
the current signatures, even on a single-threaded executor. See the
[core async contract](crates/core/README.md#building-blocks) for details.

### Advanced examples

From a checkout, run these assertion-checked examples using only `ddd_toolkit::` APIs:

| Example | Demonstrates | Command |
| --- | --- | --- |
| [Delivery retries (advanced)](examples/dispatch_retry.rs) | Aggregate, repository, dispatch failure and duplicate retry | `cargo run -p ddd-toolkit --example dispatch_retry --features derive --locked` |
| [Event sourcing](examples/event_sourcing.rs) | ExpectedRevision, upcasting, replay and snapshots | `cargo run -p ddd-toolkit --example event_sourcing --features derive --locked` |
| [Reliable delivery](examples/reliable_delivery.rs) | Async outbox, idempotent projection and checkpoints | `cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked` |

All require `derive` (enabled by default); no extra dependencies or async runtime
are needed. Adapters are in-memory demonstrations, not durable storage. Retry may
duplicate delivery; atomic transactions, persistence and claim recovery belong to
production adapters. The async demo polls only immediately-ready futures, not I/O.

### Secret values

```rust
fn main() {
    let key = ddd_toolkit::Secret::new("sekret".to_owned());
    assert_eq!(format!("{key:?}"), "[REDACTED]");
    assert_eq!(key.expose_secret(|value| value.len()), 6);
}
```

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
