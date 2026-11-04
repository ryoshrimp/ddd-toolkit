# ddd-toolkit-macro

English | [日本語](README.ja.md)

[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit-macro)](https://docs.rs/ddd-toolkit-macro)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/ddd-toolkit-macro.svg)](#license)

Derive macros for [`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit/tree/main/crates/core):
value objects, identity-based entities, domain events, and aggregate event queues.
The macros generate trait implementations and accessors; they do not provide
persistence, event delivery, or synchronization.

Maintained in the [ddd-toolkit monorepo](https://github.com/ryoshrimp/ddd-toolkit).
For development and releases, see the root repository's contribution guide.

## Installation

Tested with the latest stable Rust (edition 2024). No minimum supported Rust
version (MSRV) is specified.

```sh
cargo add ddd-toolkit-core ddd-toolkit-macro
```

Generated paths prefer a direct `ddd-toolkit-core` dependency, then fall back
to `ddd-toolkit` and its re-exports. Both dependencies may be renamed. Using
only the facade with its default `derive` feature is also supported.

## Derives

| Derive | Supported input | Generated API |
|---|---|---|
| `ValueObject` | Nonempty named or tuple structs, nonempty enums | Core `ValueObject`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, constructors, and accessors |
| `Entity` | Named struct with exactly one `#[entity(id)]` field | Core `Entity`, inherent `id()`, and identity-based `PartialEq`, `Eq`, `Hash` |
| `DomainEvent` | Struct or enum | Core `DomainEvent` marker |
| `AggregateRoot` | Named struct implementing core `Entity`, with exactly one `#[aggregate(events)]` field | Core `AggregateRoot` with `record_event` and `pull_events` |

In the default mode, do not separately implement or derive traits already generated
by a macro. Field types must satisfy the corresponding generated trait bounds.

### Trait-only mode

Use `#[value_object(trait_only)]` or struct-level `#[entity(trait_only)]` to
adopt the domain trait without replacing an existing API or trait policy:

```rust
use ddd_toolkit::{Entity, ValueObject};

#[derive(ValueObject)]
#[value_object(trait_only)]
struct Email(String);

impl Email {
    fn try_new(value: String) -> Result<Self, &'static str> {
        if value.contains('@') { Ok(Self(value)) } else { Err("missing @") }
    }
    fn as_inner(&self) -> &str { &self.0 }
}

#[derive(PartialEq, Eq, Entity)]
#[entity(trait_only)]
struct Account {
    #[entity(id)]
    id: u64,
    state: u32,
}

impl Account {
    fn id(&self) -> u64 { self.id }
}
```

- `ValueObject` generates **only** its domain trait implementation, including
  trait methods `as_inner`/`into_inner`. `Inner` follows the same shape rules as
  the default mode. No constructors (`new`, `try_new`, variant constructors),
  inherent methods, standard traits, or serde implementations are generated.
- `Entity` generates **only** its domain trait implementation (`Id` and `id`).
  Exactly one field must still have `#[entity(id)]`. No inherent `id()` or
  identity-based `PartialEq`, `Eq`, or `Hash` is generated. In this example,
  derived structural equality distinguishes accounts with the same ID but
  different state. Existing inherent methods can have different signatures;
  use `Entity::id(&account)` to explicitly call the trait method.
- You retain control of constructors, validation, getters, redacted `Debug`,
  and whether `Clone` or other standard traits exist at all. No new field trait
  bounds are added. Omitting `trait_only` preserves the existing generated API
  and semantics. Selecting individual generated methods is not supported.
- `trait_only` cannot be combined with `validate`, `error`, or `serde`; these
  combinations are errors, not silently ignored options. Keep validation and
  serialization in your own implementations.

This mode limits generated code; it does **not** guarantee invariants against
public fields, direct enum construction, or consumer-defined deserialization.
Use manual domain trait implementations instead when you need a different
`Inner`, identity mapping, or borrowing/conversion behavior.

## Value objects

For a single-field struct, `Inner` is the field type; `as_inner()` borrows that
field and `into_inner()` returns it by value. For multi-field structs and enums,
`Inner` is `Self`, so those methods borrow or return the whole value.

Structs receive `new(...)` with arguments in field declaration order.
Multi-field named structs also receive borrowed accessors named after their
fields; multi-field tuple structs receive `field_0()`, `field_1()`, and so on.

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct Quantity(u32);

#[derive(ValueObject)]
struct Address {
    city: String,
    postal_code: String,
}

#[derive(ValueObject)]
enum Status {
    Pending,
    Ready(u32),
}

fn main() {
    let quantity = Quantity::new(3);
    assert_eq!(quantity.as_inner(), &3);
    assert_eq!(quantity.into_inner(), 3);

    let address = Address::new("Tokyo".into(), "100-0001".into());
    assert_eq!(address.city(), "Tokyo");
    assert_eq!(address.as_inner(), &address);

    let status = Status::new_ready(7);
    assert!(status.is_ready());
    assert_eq!(status.as_ready(), Some((&7,)));
    assert_eq!(status.as_pending(), None);
    assert_eq!(Status::new_pending().as_pending(), Some(()));
}
```

Enums receive `new_<variant>`, `is_<variant>`, and `as_<variant>` methods.
The suffix is the fully lowercased variant name, not snake case: `InProgress`
becomes `inprogress`. A variant accessor returns `None` for another variant,
`Some(())` for a unit variant, or a tuple of field references for a data variant,
even when it has only one field. Lowercased names must be distinct, and generated
method names must not collide with other methods or field accessors.

### Validation

Specify `validate` and `error` together. The validator is a Rust path, not a
quoted string. It receives `&Inner` for a single-field struct or `&Self` for a
multi-field struct or enum, and returns `Result<(), ErrorType>`.

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(validate = validate_email, error = &'static str)]
struct Email(String);

fn validate_email(value: &String) -> Result<(), &'static str> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("missing @")
    }
}

fn main() {
    let email = Email::try_new("a@b.com".into()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(Email::try_new("invalid".into()), Err("missing @"));
}
```

Validation replaces `new` with `try_new` (or `new_<variant>` with
`try_new_<variant>`). Constructor errors are propagated unchanged; validator
panics are not caught.

Validated struct fields cannot be unrestricted `pub`. This does not prevent all
bypasses: direct construction within the defining module, restricted field
visibility, and direct enum variant construction can bypass validation.

### Core compatibility

Macro 0.5.x supports core >=0.4.1, <0.5.0. Prefer the current facade for a
coordinated combination. See the [compatibility contract](../../docs/macro-compatibility.md)
for tested versions, dependency renames and generated-code API stability.

### Serialization

Enable serde support on the **core** dependency:

```sh
cargo add ddd-toolkit-core --features serde
```

For facade-only usage, run `cargo add ddd-toolkit --features serde` instead.
If rustc reports that `serde` in `__private` is gated behind the `serde` feature,
enable that feature on **ddd-toolkit-core** (direct usage) or **ddd-toolkit**
(facade usage), not on the macro crate. When both are present the direct core
dependency takes priority. Plain derives without `#[value_object(serde)]` do
not need serde. Isolated consumer tests cover both successful configurations
and this missing-feature diagnostic.

Then opt in per value object:

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(serde)]
struct Quantity(u32);

fn main() {
    assert_eq!(Quantity::new(3).into_inner(), 3);
}
```

`#[value_object(serde)]` generates `Serialize` and `Deserialize` through the core
crate's serde re-export; it is not a feature on `ddd-toolkit-macro`. Add serde or
a format crate directly to your dependencies if your own code uses their APIs.
The attribute can be combined with `validate` and `error` in the same
`#[value_object(...)]` declaration.

Single-field structs serialize as their inner value, multi-field named structs
as structs, multi-field tuple structs as tuples, and enums as externally tagged
variants. Single-field tuple variants use Serde's newtype format (`Code(7)` →
`{"Code":7}`); other tuple variants retain tuple serialization. Container fields
keep their own shape (`Codes(vec![7, 8])` → `{"Codes":[7,8]}`).

**Breaking in macro 0.5.0 / facade 0.4.0:** older serializers added an extra tuple
wrapper, e.g. `{"Code":[7]}` or `{"Codes":[[7,8]]}`. Remove exactly that outer
array for affected variants in saved records before upgrading readers. Identify
old records using schema/version metadata rather than unwrapping every array.
No automatic legacy fallback is provided: container-valued fields can make the
formats ambiguous. Coordinate reader/writer upgrades or pin macro 0.4 / facade
0.3 until migration is ready. Other Serde formats also change from tuple to
newtype variants and need format-specific migration.

Deserialization runs the configured validator; validation errors become
`value object validation failed` rather than preserving the original error text.

## Entities and aggregate events

`Entity` compares and hashes only the field marked `#[entity(id)]`; other fields
do not participate. It provides an inherent borrowed `id()` accessor. By default,
no constructor or validation is generated.

Optional validation and serde can be used independently or together:

```rust
use ddd_toolkit::Entity;

#[derive(Entity)]
#[entity(validate = check_stock, error = &'static str, serde)]
struct Stock {
    #[entity(id)]
    id: u64,
    reserved: u32,
    total: u32,
}

fn check_stock(stock: &Stock) -> Result<(), &'static str> {
    if stock.reserved <= stock.total { Ok(()) } else { Err("over-reserved") }
}

let stock = Stock::try_new(42, 2, 5).unwrap();
assert_eq!(stock.id(), &42);
assert!(Stock::try_new(42, 6, 5).is_err());
```

- Enable the facade/core `serde` feature for `#[entity(serde)]`. No direct serde
  dependency is required for generated implementations. All fields are serialized
  in struct format, even when the ID is the only field. Field/container serde
  customization attributes are not forwarded; use manual implementations or DTOs
  when you need a different wire/storage representation.
- Specify `validate` and `error` together. The validator receives `&Self` even
  for one field. `try_new` takes every field in declaration order, including the
  ID, and propagates validation errors and panics. Without validation, no
  constructor is generated; serde-only restoration constructs the supplied state.
- Deserialization validates via `try_new` when configured. Errors become
  `entity validation failed`, without requiring `Display` on the error type.
  Neither construction nor restoration generates IDs, records creation events,
  or invokes handwritten business creation methods.
- Validated entities reject unrestricted `pub` fields. Defining-module construction,
  restricted visibility, and handwritten mutations can still bypass validation.
  State-changing methods must preserve invariants themselves. Construction checks
  are not authorization, database uniqueness checks, or transition rules.
- `trait_only` cannot be combined with `validate`, `error`, or `serde`. Existing
  handwritten `try_new` or serde implementations must not conflict with generated ones.

`DomainEvent` is a marker implementation for a struct or enum. It does not add
serialization, methods, or `Send`/`Sync` bounds.

```rust
use ddd_toolkit_core::AggregateRoot as _;
use ddd_toolkit_macro::{AggregateRoot, DomainEvent, Entity};

#[derive(DomainEvent, Debug, PartialEq)]
enum OrderEvent {
    Placed { order_id: u64 },
}

#[derive(Entity, AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    total: u32,
    #[aggregate(events)]
    events: Vec<OrderEvent>,
}

fn main() {
    let mut order = Order { id: 1, total: 10, events: Vec::new() };
    let updated = Order { id: 1, total: 20, events: Vec::new() };
    assert_ne!(order.total, updated.total);
    assert!(order == updated);
    assert_eq!(order.id(), &1);

    order.record_event(OrderEvent::Placed { order_id: 1 });
    assert_eq!(order.pull_events(), vec![OrderEvent::Placed { order_id: 1 }]);
    assert!(order.pull_events().is_empty());
}
```

The event queue must be spelled `Vec<Event>` or `std::vec::Vec<Event>`; aliases
and other vector paths are not accepted. Its event type must implement core
`DomainEvent`.

Import the core `AggregateRoot` trait to call `record_event` and `pull_events`:
they are trait methods, not inherent methods. Recording appends to the queue in
insertion order and may allocate. Pulling moves out the queued events and leaves
an empty vector. Both methods require mutable access; neither publishes events,
persists them, or updates domain state.

## Attributes

| Attribute | Placement | Effect |
|---|---|---|
| `#[value_object(validate = path, error = ErrorType)]` | `ValueObject` struct or enum | Generates fallible, validated constructors |
| `#[value_object(serde)]` | `ValueObject` struct or enum | Generates serde serialization and deserialization; requires core's `serde` feature |
| `#[entity(id)]` | One named struct field | Selects the entity identifier |
| `#[aggregate(events)]` | One named struct field | Selects the aggregate's pending event vector |

See the [API documentation](https://docs.rs/ddd-toolkit-macro) for further details
and executable examples.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
