# Learn DDD and ddd-toolkit through an order

English | [日本語](README.ja.md)

## Start here

Read **[getting_started.rs](getting_started.rs)** first. It assumes basic Rust
structs, enums, traits and `Result`, but no DDD, database or async experience.
Start with the business language, not delivery failures.

```sh
cargo run -p ddd-toolkit --example getting_started --locked
```

Expected stdout (separate from Cargo build messages):

```text
getting_started: order #1 confirmed; saved total=4200 yen; notifications=1
```

### What are we building?

A small shop creates a draft order, adds items costing 1200 and 3000 yen,
confirms the order, saves it, and notifies whoever arranges confirmation emails.
No email is actually sent: an in-memory notification lets us check the result.

The business rules are: prices must be positive, an empty order cannot be
confirmed, and a confirmed order cannot be changed or confirmed again.
`Order::add_item` and `Order::confirm` own these rules so that another caller
cannot accidentally skip validation. Derives supply traits, **not business rules**;
those are your responsibility.

### Vocabulary mapped to code

| Term | Meaning in this example | Code |
| --- | --- | --- |
| Domain | The business and its rules | Order confirmation rules |
| Entity | Something identified by ID even when its contents change | `Order`, `id` |
| Aggregate / aggregate root | A consistency boundary / its entry point for changes | `Order`, private fields and business methods |
| Repository | A contract for saving and retrieving aggregates | `Repository`, `MemoryOrders` |
| Domain event | A business fact that happened, not a command | `OrderConfirmed` |
| Application layer | Coordinates business methods, persistence and notification | `main` |
| Adapter | Connects a contract to a concrete storage or notification destination | `MemoryOrders`, `Notifications` |

DDD means modeling business language, boundaries and rules, not using every trait
in this crate. An aggregate can contain multiple entities; ours contains just one.
Value objects (types distinguished by value rather than identity) and multiple
aggregates are outside this introduction, as are production money types, taxes,
line items, stock, shipping and payments.

### Reading order

1. **`OrderError` and the `Order` methods**: find the business rules. Private fields
   keep consumers from directly setting `confirmed`. `main` is in the same file
   and reads state for assertions, but changes it through the methods.
2. **The first half of `main`**: reject an empty order, add items, confirm.
   Assertions check results; they do not implement the business rules.
3. **Save, then notify**: `save` stores order state. `record_event` only queues a
   notification locally. Only `dispatch_events` invokes `publish`, clearing the
   pending queue on success. Saving state is not saving event history: this
   example does **not** use event sourcing.
4. **Adapters at the bottom**: come back to these last. `Infallible` means this
   memory adapter has no adapter errors, not that a real database cannot fail.

Try changing item prices or the order of calls to `confirm`. Assertions verify
that empty confirmation, zero prices and post-confirmation changes are rejected
without changing the total or emitting extra events. `unwrap` keeps this fixed
scenario readable; applications with user input or external I/O need to handle
errors, for example by returning them to the caller.

### Where to go next

| Order | Example | Lesson / prerequisites |
| --- | --- | --- |
| 1 | [getting_started.rs](getting_started.rs) | Business rules, persistence, notification. Enough to start |
| 2 | [cqrs.rs](cqrs.rs) | Separate update commands from read queries. A Specification tests a condition; a Policy enforces a constraint with a rejection reason |
| 3 | [basic.rs](basic.rs) | **Advanced: partial delivery failure and duplicates**. Despite the name, not the first tutorial |
| Optional | [event_sourcing.rs](event_sourcing.rs) | **Advanced: reconstruct state from facts**. Revisions, snapshots and upcasters |
| Optional | [reliable_delivery.rs](reliable_delivery.rs) | **Advanced: async and failure recovery**. Outbox, projection, checkpoint and deduplication |

Each file is an independent program. A later `Order` does not extend the tutorial
model: later examples deliberately narrow the model to focus on API contracts.
`cqrs` and `basic` share [support/mod.rs](support/mod.rs). The CQRS example adjusts
an order total; it is not the tutorial's confirmation workflow. CQRS separates
update and read responsibilities; it does not require another database or async.
A lifecycle callback makes save/dispatch ordering explicit, not transactional.

## Run all examples and check results

Run these commands from the repository root. All examples use the facade only,
require the `derive` feature (on by default), and assert their expected results.
No new dependency, database, broker, or async runtime is required.

```sh
cargo build -p ddd-toolkit --examples --all-features --locked
cargo run -p ddd-toolkit --example getting_started --features derive --locked
cargo run -p ddd-toolkit --example basic --features derive --locked
cargo run -p ddd-toolkit --example cqrs --features derive --locked
cargo run -p ddd-toolkit --example event_sourcing --features derive --locked
cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked
```

- [getting_started.rs](getting_started.rs): business rules → confirmation → save →
  notify. Total 4200 yen, one notification. Rejects empty orders, zero prices and
  changes after confirmation.
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

## Vocabulary for the advanced examples

- **Revision / ExpectedRevision**: a position in a stream (one aggregate's event
  sequence) / the position expected before a write. Rejecting a stale write
  prevents overwriting a concurrent change.
- **Replay / snapshot / upcaster**: rebuilding state by applying past events / a
  cached intermediate state / a conversion from an old event format to the current
  format. The Counter deliberately illustrates only these mechanics.
- **Outbox**: pending notifications stored in the same transaction as state.
- **Projection**: read-side state built from events; here, a balance from credits.
- **Checkpoint**: a reader's processing position. A position alone cannot prevent
  double application; stable IDs and deduplication are also needed here.
- **At-least-once**: delivery may repeat a notification. Idempotent application
  prevents repeated credits from increasing the balance more than once.

Event notification is not event sourcing. Publishing a notification does not
require rebuilding state from history or adopting an outbox. Choose these only
when needed.

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

CI explicitly builds and runs all five examples, in addition to the normal
workspace tests and feature matrix. `required-features = ["derive"]` skips these
derive-based targets during no-default-features builds.
