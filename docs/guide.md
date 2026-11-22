# Getting-started guide

English | [日本語](guide.ja.md)

ddd-toolkit is meant to be adopted in stages. **Stage 1 alone is enough for many
applications; every later stage is optional.** Move on only when you hit the
problem the "Move on when" hint describes. For the complete list of public items
grouped by area, see the API map in the [crate documentation](https://docs.rs/ddd-toolkit)
(it appears on docs.rs from the next release; until then see `src/lib.rs` or run `cargo doc --open`);
for per-API examples and contract tests, see the
[public contract map](public-contracts.md).

All commands run from the repository root.

## Stage 1: Minimal

**You can:** express business rules in value objects, entities and an aggregate
root; record domain events inside the aggregate; save and load it through a
repository.

**APIs:** `ValueObject`, `Entity`, `AggregateRoot`, `DomainEvent`, `Repository`
(`AsyncRepository` for async storage).

**Read and run:** the [order tutorial](../examples/README.md#start-here) and
[getting_started.rs](../examples/getting_started.rs).

```sh
cargo run -p ddd-toolkit --example getting_started --features derive --locked
```

**Move on when** other code must react to the recorded events.

## Stage 2: Event publication

**You can:** publish the events an aggregate recorded and clear them once
delivery succeeds. Retry behavior after partial failure is part of this stage.

**APIs:** `DomainEventPublisher`, `dispatch_events`; async counterparts
`AsyncDomainEventPublisher`, `dispatch_events_async`.

**Read and run:** [dispatch_retry.rs](../examples/dispatch_retry.rs).

```sh
cargo run -p ddd-toolkit --example dispatch_retry --features derive --locked
```

**Move on when** you want to separate update operations from reads, or make the
save-then-dispatch order explicit.

## Stage 3: CQRS

**You can:** separate commands from queries, run save and dispatch inside a
lifecycle context, and reject invalid input with `Specification` (tests a
condition) and `Policy` (enforces a constraint with a reason).

**APIs:** `CommandHandler`, `QueryHandler`, `AggregateCreator`,
`LifecycleContext` and related types, `Specification`, `Policy` (async
counterparts exist for each handler and context; `AggregateCreator` is not used in
the example, see [public-contracts.md](public-contracts.md)).

**Read and run:** [cqrs.rs](../examples/cqrs.rs).

```sh
cargo run -p ddd-toolkit --example cqrs --features derive --locked
```

**Move on when** you need the full history of changes, not only the latest state.

## Stage 4: Event sourcing

**You can:** store events as the source of truth, guard appends with expected
revisions, rebuild aggregates by replay, shorten replay with snapshots, and
evolve old event versions with upcasters.

**APIs:** `EventStore` (`AsyncEventStore`), `ExpectedRevision`, `StreamRevision`
and the other revision types, `AggregateReplay`, `replay_aggregate`,
`SnapshotStore`, `AggregateSnapshot`, `EventUpcaster`, `UpcasterChain` (`UpcasterChain` is not used
in the example, see [public-contracts.md](public-contracts.md)).

**Read and run:** [event_sourcing.rs](../examples/event_sourcing.rs).

```sh
cargo run -p ddd-toolkit --example event_sourcing --features derive --locked
```

**Move on when** events must reach other systems or read models reliably.

## Beyond

- **Reliable delivery:** transactional outbox, event consumers, projections,
  checkpoints and idempotent projections. See
  [reliable_delivery.rs](../examples/reliable_delivery.rs)
  (`cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked`).
- **Other building blocks:** `DomainService`, `DomainFactory`, `ProcessManager`,
  `UnitOfWork`. See [public-contracts.md](public-contracts.md) for their
  examples and contract tests.
- **Secrets:** `Secret` redacts formatting and zeroizes on drop; it needs no
  feature flag. See [secret-serde.md](secret-serde.md) for serialization.
