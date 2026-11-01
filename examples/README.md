# Runnable facade examples

English | [日本語](README.ja.md)

Run these commands from the repository root. All examples use the facade only,
require the `derive` feature (on by default), and assert their expected results.
No new dependency, database, broker, or async runtime is required.

```sh
cargo build -p ddd-toolkit --examples --all-features --locked
cargo run -p ddd-toolkit --example basic --features derive --locked
cargo run -p ddd-toolkit --example cqrs --features derive --locked
cargo run -p ddd-toolkit --example event_sourcing --features derive --locked
cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked
```

- [basic.rs](basic.rs): saves an order, fails after publishing the first event,
  then retries the restored batch. Expected deliveries: `[1, 1, 2]`. Saving does
  not drain pending events; publication failure does not roll back the save.
- [cqrs.rs](cqrs.rs): rejects zero increments (Specification) and totals above
  5000 (Policy), then saves and dispatches using an explicit lifecycle callback.
  The query returns 4200. This simple query reads repository state; CQRS does not
  require a separate database. Save and publish are still separate operations.
- [event_sourcing.rs](event_sourcing.rs): loads a legacy v1 event through an
  upcaster, replays it, saves a snapshot, then appends using `ExpectedRevision`.
  A stale append is rejected without mutation. Snapshot plus tail and full replay
  both produce 5 at revision 1 (revisions are zero-based). Upcasting belongs to
  the adapter's load boundary, and replay never records pending events. This
  example retains full history even when using a snapshot; it does not compact it.
- [reliable_delivery.rs](reliable_delivery.rs): saves an account and enqueues an
  event through the async outbox lifecycle. Broker, checkpoint-save, and
  acknowledgement failures leave the claim retryable. Three successful broker
  deliveries produce only one projection update (balance 10), thanks to a stable
  message ID. The checkpoint is saved after applying and before acknowledging;
  if saving fails, the retry is `AlreadyApplied` and still saves the checkpoint.
  The demo directly invokes the projection rather than running a broker consumer.

## Adapter boundaries

All storage and deduplication state is **in memory** and lost on process exit.
These are small application adapters, not built-in toolkit implementations or
production durability guarantees. In production:

- Persist aggregate state and outbox entries atomically in an adapter transaction;
  optimistic revision checks and append must also be atomic. A lifecycle context
  alone cannot provide database transactions, rollback, or crash recovery.
- Specify claim leases, concurrent-worker ownership, recovery and retry/backoff.
  The demo has a single account/worker and reclaims the queue head until ack.
- Assume at-least-once delivery: success followed by an ack failure can duplicate
  publication. Persist projection changes and deduplication markers atomically.
  If a separate checkpoint save fails, replay must not apply the effect twice.
- Choose durable cursor semantics and ordering. This demo uses stable message IDs
  as cursors, not sortable stream offsets; it never compares them for ordering.
- Async contracts return boxed `Send` futures without selecting a runtime. The
  demo's bounded `ready` helper polls once and rejects `Pending`; it is **not**
  a general executor. Use an application-selected executor for real I/O, and
  design cancellation/error recovery in the adapters. The outbox lifecycle
  consumes its event batch even on failure/cancellation; blindly repeating a
  command or enqueue operation can lose or duplicate effects. Resolve uncertain
  commits through durable application/adapter recovery rather than this helper.

CI explicitly builds and runs all four examples, in addition to the normal
workspace tests and feature matrix. `required-features = ["derive"]` skips these
derive-based targets during no-default-features builds.
