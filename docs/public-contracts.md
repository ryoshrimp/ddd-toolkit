# Public contracts, examples and contract tests / 公開契約・実行例・契約テスト

## English

This index covers the user-facing items re-exported by the
[facade](../src/lib.rs) from [core](../crates/core/src/lib.rs), plus the four
derive macros. Names in the tables are available as `ddd_toolkit::<name>`.
The hidden `__private` module is generated-code infrastructure, not a user-facing
contract. See [API documentation](https://docs.rs/ddd-toolkit) for signatures.

- **Examples** are runnable, assertion-checked application flows using local
  in-memory adapters. Start with the [example guide](../examples/README.md).
- **Contract tests** check individual boundaries, errors and ordering, including
  contracts with no runnable example. A test adapter demonstrates the contract;
  it does not provide a production storage or delivery implementation.
- **Mode:** `S` = synchronous operations, `A` = asynchronous operations,
  `—` = shared data, marker or composition contract (not an async port).
  Async ports use boxed `Send` futures; no async runtime is supplied.
- **Features:** every core trait/helper in the tables is available with
  `default-features = false`, including async contracts and `Secret`.
  Only the four derive macros require `derive` (enabled by default).
  `serde` is optional; generated value-object serialization additionally needs
  `#[value_object(serde)]`. It is not a prerequisite for the event contracts.
- Every linked runnable example currently requires `derive` (see
  [Cargo.toml](../Cargo.toml)). This does not make `derive` a requirement of
  the traits those examples implement. Rows saying **Tests only** have no
  corresponding runnable example today.

From the repository root:

```sh
cargo run -p ddd-toolkit --example getting_started --locked
# Replace getting_started with dispatch_retry, cqrs, event_sourcing or reliable_delivery.
cargo test -p ddd-toolkit-core --test generic_capabilities --locked
# Replace generic_capabilities with a linked core test file's stem.
cargo test -p ddd-toolkit-facade-tests --locked
```

When adding or changing an example or public re-export, update the relevant row
and keep its mode, feature requirements and test links accurate.

## 日本語

この対応表は[ファサード](../src/lib.rs)が[core](../crates/core/src/lib.rs)から
再公開する利用者向けの項目と、四つの derive マクロを対象にします。
表中の名前は `ddd_toolkit::<名前>` で利用できます。非表示の `__private` は
生成コード用の内部基盤であり、利用者向け契約には含めません。
シグネチャは [API ドキュメント](https://docs.rs/ddd-toolkit)を参照してください。

- **実行例（Example）**は、メモリ内アダプターを使い、assert で動作を確認する
  アプリケーションの流れです。[実行例ガイド](../examples/README.ja.md)から始めてください。
- **契約テスト（Contract tests）**は、個別の境界・エラー・順序を検証します。
  実行例のない契約も案内します。テスト用アダプターは本番用の永続化や配送実装ではありません。
- **方式（Mode）:** `S` は同期、`A` は非同期、`—` は共通のデータ・マーカー・
  合成契約（非同期ポートではない）です。非同期ポートはボックス化された `Send` Future を
  返し、ランタイムは提供しません。
- **feature:** 表中の core トレイト・ヘルパーはすべて `default-features = false` で
  利用でき、非同期契約や `Secret` も対象です。四つのマクロだけが `derive`（既定で有効）を
  必要とします。`serde` は任意で、値オブジェクトの生成された直列化には型ごとの
  `#[value_object(serde)]` も必要です。イベント契約の前提条件ではありません。
- 現在の実行例はすべて `derive` が必要です（[Cargo.toml](../Cargo.toml)参照）。
  実行例で実装するトレイト自体に `derive` が必要なわけではありません。
  **Tests only / テストのみ** の行は、現時点で対応する実行例がありません。

上記コマンドはリポジトリのルートで実行します。実行例名や core テストのファイル名
（拡張子なし）を置き換えると、各リンク先を実行できます。実行例や公開する項目を
追加・変更した際は、該当行・同期／非同期・feature 条件・テストへのリンクも更新してください。

## Domain modeling / ドメインモデル

| Public items / 公開項目 | Mode / 方式 | Example / 実行例 | Contract tests / 契約テスト |
| --- | --- | --- | --- |
| `Entity`, `ValueObject`, `AggregateRoot`, `DomainEvent` (traits / トレイト) | — | [getting_started](../examples/getting_started.rs) | [domain_flow](../facade-tests/tests/domain_flow.rs), [domain_events](../crates/core/tests/domain_events.rs) |
| `Entity`, `ValueObject`, `AggregateRoot`, `DomainEvent` (derive macros; `derive` feature) | — | [getting_started](../examples/getting_started.rs) | [derive_via_facade](../facade-tests/tests/derive_via_facade.rs); optional `serde`: [serde_via_facade](../facade-tests/tests/serde_via_facade.rs) |
| `Secret`, `SecretMetadata` | — | Tests only / テストのみ | [secret](../crates/core/tests/secret.rs), [secret_via_facade](../facade-tests/tests/secret_via_facade.rs), [secret_boundary_compile_fail](../crates/core/tests/secret_boundary_compile_fail.rs) |
| `Specification`, `Policy` | S | [cqrs](../examples/cqrs.rs) | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) |
| `And`, `Or`, `Not` | S | Tests only / テストのみ | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) |
| `DomainService`, `DomainFactory` | S | Tests only / テストのみ | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) |
| `ProcessManager`, `Transition` | S / — | Tests only / テストのみ | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) (`process_manager_is_deterministic_and_reentrant`) |
| `UnitOfWork` | S | Tests only / テストのみ | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) (`unit_of_work_exposes_explicit_lifecycle`) |

## Persistence, publishing and CQRS / 永続化・発行・CQRS

| Public items / 公開項目 | Mode / 方式 | Example / 実行例 | Contract tests / 契約テスト |
| --- | --- | --- | --- |
| `Repository` | S | [getting_started](../examples/getting_started.rs) | [repository](../crates/core/tests/repository.rs), [domain_flow](../facade-tests/tests/domain_flow.rs) |
| `AsyncRepository` | A | Tests only / テストのみ | [async_contracts](../crates/core/tests/async_contracts.rs), [async_via_facade](../facade-tests/tests/async_via_facade.rs) |
| `DomainEventPublisher`, `dispatch_events` | S | [dispatch_retry](../examples/dispatch_retry.rs) | [domain_event_publishing](../crates/core/tests/domain_event_publishing.rs), [aggregate_event_dispatch](../crates/core/tests/aggregate_event_dispatch.rs) |
| `AsyncDomainEventPublisher` | A | [reliable_delivery](../examples/reliable_delivery.rs) | [async_contracts](../crates/core/tests/async_contracts.rs) |
| `dispatch_events_async` | A | Tests only / テストのみ | [async_contracts](../crates/core/tests/async_contracts.rs) |
| `DomainEventMetadata`, `DomainEventConversion` | S | Tests only / テストのみ | [domain_event_practicalization](../crates/core/tests/domain_event_practicalization.rs), [domain_event_contract_compile_fail](../crates/core/tests/domain_event_contract_compile_fail.rs) |
| `AggregateCreator` | S | Tests only / テストのみ | [cqrs](../crates/core/tests/cqrs.rs) |
| `CommandHandler`, `QueryHandler`, `LifecycleCallback`, `LifecycleContext` | S | [cqrs](../examples/cqrs.rs) | [cqrs](../crates/core/tests/cqrs.rs), [cqrs_compile_pass](../crates/core/tests/cqrs_compile_pass.rs), [cqrs_compile_fail](../crates/core/tests/cqrs_compile_fail.rs) |
| `AsyncCommandHandler`, `AsyncAggregateCreator`, `AsyncQueryHandler`, `AsyncLifecycleCallback`, `AsyncLifecycleContext` | A | Tests only / テストのみ | [async_cqrs](../crates/core/tests/async_cqrs.rs), [async_via_facade](../facade-tests/tests/async_via_facade.rs) |
| `LifecycleCompletionError`, `LifecycleOperationError` | — (S/A) | [cqrs](../examples/cqrs.rs) | [cqrs](../crates/core/tests/cqrs.rs), [async_cqrs](../crates/core/tests/async_cqrs.rs) |

## Event sourcing / イベントソーシング

| Public items / 公開項目 | Mode / 方式 | Example / 実行例 | Contract tests / 契約テスト |
| --- | --- | --- | --- |
| `EventStore` | S | [event_sourcing](../examples/event_sourcing.rs) | [event_sourcing_contracts](../crates/core/tests/event_sourcing_contracts.rs), [event_sourcing_compile_fail](../crates/core/tests/event_sourcing_compile_fail.rs) |
| `AsyncEventStore`, `AsyncSnapshotStore` | A | Tests only / テストのみ | [eventing_ports](../crates/core/tests/eventing_ports.rs) |
| `AppendError`, `AppendResult`, `CurrentRevision`, `ExpectedRevision`, `LoadResult`, `NoStream`, `RevisionConflict`, `StreamHistory`, `StreamHistoryError`, `StreamRevision` | — | [event_sourcing](../examples/event_sourcing.rs) | [event_sourcing_contracts](../crates/core/tests/event_sourcing_contracts.rs), [reliability_helpers](../crates/core/tests/reliability_helpers.rs) |
| `AggregateReplay`, `RehydratedAggregate`, `ReplayError`, `SnapshotReplayError`, `replay_aggregate`, `replay_with_optional_snapshot` | S / — | [event_sourcing](../examples/event_sourcing.rs) | [event_sourcing_contracts](../crates/core/tests/event_sourcing_contracts.rs) |
| `SnapshotStore`, `AggregateSnapshot`, `SnapshotLoadError`, `SnapshotSaveError`, `SnapshotUse` | S / — | [event_sourcing](../examples/event_sourcing.rs) | [event_sourcing_snapshots](../crates/core/tests/event_sourcing_snapshots.rs) |
| `EventUpcaster` | S | [event_sourcing](../examples/event_sourcing.rs) | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) |
| `EventIdentity`, `VersionedEvent`, `UpcasterChain` | S / — | Tests only / テストのみ | [generic_capabilities](../crates/core/tests/generic_capabilities.rs) (`version_identity_and_upcasters_are_ordered`) |

## Delivery and projections / 配送・プロジェクション

| Public items / 公開項目 | Mode / 方式 | Example / 実行例 | Contract tests / 契約テスト |
| --- | --- | --- | --- |
| `EventConsumer`, `Projection`, `AsyncEventConsumer`, `AsyncProjection` | S / A | Tests only / テストのみ | [eventing_ports](../crates/core/tests/eventing_ports.rs) |
| `DeliveryMetadata`, `DomainEventDeliveryMetadata` | — / S | Tests only / テストのみ | [eventing_ports](../crates/core/tests/eventing_ports.rs), [reliability_helpers](../crates/core/tests/reliability_helpers.rs) |
| `TransactionalOutbox` | S | Tests only / テストのみ | [eventing_ports](../crates/core/tests/eventing_ports.rs), [outbox_lifecycle](../crates/core/tests/outbox_lifecycle.rs) |
| `AsyncTransactionalOutbox` | A | [reliable_delivery](../examples/reliable_delivery.rs) | [async_transactional_outbox](../crates/core/tests/async_transactional_outbox.rs) |
| `OutboxClaim`, `OutboxMessageId` | — (S/A) | [reliable_delivery](../examples/reliable_delivery.rs) | [eventing_ports](../crates/core/tests/eventing_ports.rs), [reliability_helpers](../crates/core/tests/reliability_helpers.rs) |
| `TransactionalOutboxLifecycleContext`, `OutboxLifecycleContext` (alias / 別名) | S | Tests only / テストのみ | [outbox_lifecycle](../crates/core/tests/outbox_lifecycle.rs) |
| `TransactionalOutboxCompletionError`, `TransactionalOutboxOperationError`, `OutboxLifecycleCompletionError`, `OutboxLifecycleOperationError` (aliases / 別名を含む) | — (S) | Tests only / テストのみ | [outbox_lifecycle](../crates/core/tests/outbox_lifecycle.rs) |
| `AsyncTransactionalOutboxLifecycleContext`, `AsyncOutboxLifecycleContext` (alias / 別名) | A | [reliable_delivery](../examples/reliable_delivery.rs) | [async_transactional_outbox](../crates/core/tests/async_transactional_outbox.rs) |
| `AsyncTransactionalOutboxCompletionError`, `AsyncTransactionalOutboxOperationError`, `AsyncOutboxLifecycleCompletionError`, `AsyncOutboxLifecycleOperationError` (aliases / 別名を含む) | — (A) | [reliable_delivery](../examples/reliable_delivery.rs) | [async_transactional_outbox](../crates/core/tests/async_transactional_outbox.rs) |
| `ProjectionCheckpoint`, `ProjectionApplication` | — (S/A) | [reliable_delivery](../examples/reliable_delivery.rs) | [projection_reliability](../crates/core/tests/projection_reliability.rs) |
| `ProjectionCheckpointStore`, `IdempotentProjection` | S | Tests only / テストのみ | [projection_reliability](../crates/core/tests/projection_reliability.rs) |
| `AsyncProjectionCheckpointStore`, `AsyncIdempotentProjection` | A | [reliable_delivery](../examples/reliable_delivery.rs) | [projection_reliability](../crates/core/tests/projection_reliability.rs) |

## Guarantees and boundaries / 保証と境界

Examples and tests do not turn ports into infrastructure: applications supply
storage, transactions, runtime execution, checkpoint coordination and duplicate
detection. Dispatch restoration does not undo publication side effects; retries
can deliver duplicates. Outbox lifecycle contexts track one attempt, not a database
transaction; dropping their async future does not restore pulled events. See the
linked tests and API documentation for error and cancellation boundaries.

実行例やテストがあっても、ポートがインフラを実装するわけではありません。保存・
トランザクション・ランタイム・チェックポイントとの連携・重複検出はアプリケーション側の
責任です。dispatch のイベント復元は発行の副作用を取り消さず、再試行で重複配送が
起こり得ます。outbox のライフサイクルコンテキストは一回の試行を追跡するもので、
DB トランザクションではありません。その非同期 Future を破棄しても、取り出した
イベントは復元されません。エラーとキャンセルの境界はリンク先のテストと API 文書を参照してください。
