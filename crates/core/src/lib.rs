//! **English:**<br>
//! Core traits for modeling Domain-Driven Design entities and value objects.
//!
//! The crate root holds the central patterns, which are the stable surface: [`ValueObject`],
//! [`Entity`], [`AggregateRoot`], [`DomainEvent`] with [`DomainEventMetadata`] and
//! [`DomainEventConversion`], [`Repository`] and [`AsyncRepository`], [`DomainEventPublisher`] and
//! [`AsyncDomainEventPublisher`] with [`dispatch_events`] and [`dispatch_events_async`],
//! [`UnitOfWork`], [`Specification`] with [`And`], [`Or`], and [`Not`], and [`DomainService`],
//! [`Policy`], and [`DomainFactory`]. [`Entity`] exposes a stable identifier by reference, while
//! [`ValueObject`] exposes a wrapped value by reference or by value. Implement these traits on
//! domain types to keep their domain-facing APIs independent from persistence details.
//!
//! Everything else lives in public modules that are experimental in 0.x and may change, each
//! behind a Cargo feature that is enabled by default:
//!
//! - `cqrs` (feature `cqrs`): command and query handlers and their lifecycle contexts.
//! - `event_sourcing` (feature `event_sourcing`): event stores, replay, snapshots, and event
//!   versioning.
//! - `delivery` (feature `delivery`, which enables `event_sourcing`): transactional outboxes,
//!   event consumers, and projections.
//! - `process` (feature `process`): process managers.
//! - `secret` (feature `secret`): `Secret` and related types; the `serde` feature adds
//!   deserialization when `secret` is also enabled, and `serde_with` enables `secret`, `serde`,
//!   and the explicit `ExposeSecret` output adapter.
//!
//! The thin traits ([`DomainEvent`], [`DomainService`], [`Policy`], [`DomainFactory`], and
//! [`Specification`]) add no behavior of their own, apart from the combinators ([`And`], [`Or`],
//! [`Not`]) that [`Specification`] provides. They help when code is generic over a role
//! (for example `P: Policy<Order>`) instead of a concrete type, when tests swap in a stub
//! implementation, and when signatures and docs should name the domain role. They are optional:
//! if a rule or operation is used in one place and never swapped, a plain function or inherent
//! method is enough. They provide no transaction, retry, persistence, or thread-safety behavior.
//! The example at the end of this page shows a [`Policy`] used as a bound and replaced by a stub.
//!
//! Every async port returns `Pin<Box<dyn Future + Send + 'a>>`. Implementations whose futures are
//! `!Send` (for example on WASM or single-threaded executors that hold `Rc` or `RefCell` across
//! `.await`) do not fit these signatures; there are no non-`Send` variants. Moving to native
//! `async fn` in traits is deferred until return type notation is stable, so that callers, not the
//! trait, can choose whether to require `Send`.
//!
//! **日本語:**<br>
//! ドメイン駆動設計のエンティティと値オブジェクトを表現するためのコアトレイトを提供します。
//!
//! クレートのルートには中心となるパターンを置き、これが安定させる対象です。[`ValueObject`]、
//! [`Entity`]、[`AggregateRoot`]、[`DomainEvent`]（[`DomainEventMetadata`] と
//! [`DomainEventConversion`] を含む）、[`Repository`] と [`AsyncRepository`]、
//! [`DomainEventPublisher`] と [`AsyncDomainEventPublisher`]（[`dispatch_events`] と
//! [`dispatch_events_async`] を含む）、[`UnitOfWork`]、[`Specification`]（[`And`]、[`Or`]、
//! [`Not`] を含む）、[`DomainService`]、[`Policy`]、[`DomainFactory`] です。[`Entity`] は安定した
//! 識別子を参照で公開し、[`ValueObject`] は内包値を参照または値で公開します。永続化の詳細から
//! 独立したドメイン型の API を保つために、これらのトレイトを実装します。
//!
//! それ以外は公開モジュールに置きます。0.x では実験的で変わりうる扱いで、いずれも既定で
//! 有効な Cargo feature で切り替えます。
//!
//! - `cqrs`（feature `cqrs`）: コマンド・クエリのハンドラーとライフサイクルコンテキスト。
//! - `event_sourcing`（feature `event_sourcing`）: イベントストア、復元、スナップショット、
//!   イベントのバージョニング。
//! - `delivery`（feature `delivery`。`event_sourcing` を有効にします）: トランザクショナル
//!   アウトボックス、イベントコンシューマー、プロジェクション。
//! - `process`（feature `process`）: プロセスマネージャー。
//! - `secret`（feature `secret`）: `Secret` と関連型。`secret` と併せて `serde` feature で
//!   復元に対応し、`serde_with` は `secret`、`serde`、明示的な出力アダプター `ExposeSecret` を
//!   有効にします。
//!
//! 薄いトレイト（[`DomainEvent`]、[`DomainService`]、[`Policy`]、[`DomainFactory`]、
//! [`Specification`]）は、[`Specification`] の結合操作（[`And`]、[`Or`]、[`Not`]）を除き、
//! 固有の動作を追加しません。具体型ではなく役割（例: `P: Policy<Order>`）に
//! 対してジェネリックに書くとき、テストでスタブ実装に差し替えるとき、シグネチャやドキュメントで
//! ドメイン上の役割を型名で明示したいときに役立ちます。利用は任意で、1 か所でしか使わず
//! 差し替えない規則や操作なら、通常の関数や固有メソッドで十分です。トランザクション、再試行、
//! 永続化、スレッド安全性の動作は提供しません。
//! 下の使用例は、[`Policy`] をジェネリックな境界として使い、スタブに差し替える様子を示します。
//!
//! 非同期ポートはすべて `Pin<Box<dyn Future + Send + 'a>>` を返します。Future が `!Send` になる実装
//! （たとえば WASM や、`.await` をまたいで `Rc` や `RefCell` を保持する単一スレッド executor 向けの実装）は
//! これらのシグネチャに適合せず、`Send` を要求しない版もありません。`Send` を要求するかどうかを trait ではなく
//! 呼び出し側が選べるように、ネイティブの `async fn` in trait への移行は return type notation の安定化まで見送ります。
//!
//! # Example / 使用例
//!
//! ```rust
//! use ddd_toolkit_core::Policy;
//!
//! struct Order {
//!     total: u32,
//! }
//!
//! // The application service depends on the role, not on a concrete policy.
//! // アプリケーションサービスは具体的なポリシーではなく役割に依存します。
//! struct PlaceOrder<P> {
//!     policy: P,
//! }
//!
//! impl<P: Policy<Order>> PlaceOrder<P> {
//!     fn place(&self, order: &Order) -> Result<u32, P::Error> {
//!         self.policy.check(order)?;
//!         Ok(order.total)
//!     }
//! }
//!
//! struct MaxTotal(u32);
//! impl Policy<Order> for MaxTotal {
//!     type Error = String;
//!     fn check(&self, order: &Order) -> Result<(), String> {
//!         if order.total <= self.0 {
//!             Ok(())
//!         } else {
//!             Err(format!("total {} exceeds {}", order.total, self.0))
//!         }
//!     }
//! }
//!
//! // A test stub swapped in for the real policy.
//! // 実際のポリシーの代わりに差し替えるテスト用スタブです。
//! struct AlwaysReject;
//! impl Policy<Order> for AlwaysReject {
//!     type Error = &'static str;
//!     fn check(&self, _: &Order) -> Result<(), &'static str> {
//!         Err("rejected")
//!     }
//! }
//!
//! let order = Order { total: 100 };
//! assert_eq!(PlaceOrder { policy: MaxTotal(500) }.place(&order), Ok(100));
//! assert_eq!(PlaceOrder { policy: AlwaysReject }.place(&order), Err("rejected"));
//! ```
#![deny(unsafe_code)]

// Generated derives use the library name in both library and consumer targets.
extern crate self as ddd_toolkit_core;

mod aggregate_root;
mod async_dispatch;
mod async_domain_event_publisher;
#[cfg(feature = "delivery")]
mod async_event_consumption;
#[cfg(feature = "event_sourcing")]
mod async_event_store;
#[cfg(feature = "delivery")]
mod async_outbox_lifecycle;
mod async_repository;
#[cfg(feature = "event_sourcing")]
mod async_snapshot;
#[cfg(feature = "delivery")]
mod async_transactional_outbox;
// The public `cqrs` module below re-exports this one.
#[cfg(feature = "cqrs")]
#[path = "cqrs/mod.rs"]
mod cqrs_impl;
mod dispatch;
mod domain_event;
mod domain_event_conversion;
mod domain_event_metadata;
mod domain_event_publisher;
mod domain_factory;
mod domain_service;
mod entity;
#[cfg(feature = "delivery")]
mod event_consumption;
#[cfg(feature = "delivery")]
mod event_delivery_metadata;
mod event_restoration_guard;
#[cfg(feature = "event_sourcing")]
mod event_store;
#[cfg(feature = "event_sourcing")]
mod event_versioning;
#[cfg(feature = "serde_with")]
mod expose_secret;
#[cfg(feature = "delivery")]
mod outbox_lifecycle;
#[cfg(feature = "process")]
mod process_manager;
#[cfg(feature = "delivery")]
mod projection_reliability;
#[cfg(feature = "event_sourcing")]
mod replay;
mod repository;
// The public `secret` module below re-exports this one.
#[cfg(feature = "secret")]
#[path = "secret.rs"]
mod secret_impl;
#[cfg(feature = "secret")]
mod secret_metadata;
#[cfg(feature = "event_sourcing")]
mod snapshot;
mod specification;
#[cfg(feature = "delivery")]
mod transactional_outbox;
mod unit_of_work;
mod value_object;

pub use aggregate_root::AggregateRoot;

pub use async_dispatch::dispatch_events_async;

pub use async_domain_event_publisher::AsyncDomainEventPublisher;

pub use async_repository::AsyncRepository;

pub use dispatch::dispatch_events;

pub use domain_event::DomainEvent;

pub use domain_event_conversion::DomainEventConversion;

pub use domain_event_metadata::DomainEventMetadata;

pub use domain_event_publisher::DomainEventPublisher;

pub use domain_factory::DomainFactory;

pub use domain_service::{DomainService, Policy};

pub use entity::Entity;

pub use repository::Repository;

pub use specification::{And, Not, Or, Specification};

pub use unit_of_work::UnitOfWork;

pub use value_object::ValueObject;

#[cfg(feature = "cqrs")]
pub mod cqrs {
    //! **English:**<br>
    //! CQRS command and query handlers with lifecycle coordination.
    //!
    //! This module is experimental: it may change in 0.x releases. It requires the `cqrs` feature,
    //! which is enabled by default.
    //!
    //! [`CommandHandler`] and [`AsyncCommandHandler`] handle commands, [`QueryHandler`] and
    //! [`AsyncQueryHandler`] handle queries, and [`AggregateCreator`] and
    //! [`AsyncAggregateCreator`] create aggregates. [`LifecycleContext`] and
    //! [`AsyncLifecycleContext`] require a successful save and event dispatch before
    //! completion, reporting failures through [`LifecycleCompletionError`] and
    //! [`LifecycleOperationError`]; [`LifecycleCallback`] and [`AsyncLifecycleCallback`]
    //! adapt callbacks. A failed attempt is consumed and cannot be retried through the same
    //! context.
    //!
    //! **日本語:**<br>
    //! CQRS のコマンド・クエリのハンドラーとライフサイクル調整機能を提供します。
    //!
    //! このモジュールは実験的で、0.x のリリースで変わる可能性があります。feature `cqrs`
    //! （既定で有効）が必要です。
    //!
    //! [`CommandHandler`] と [`AsyncCommandHandler`] はコマンドを、[`QueryHandler`] と
    //! [`AsyncQueryHandler`] はクエリを処理し、[`AggregateCreator`] と [`AsyncAggregateCreator`] は
    //! 集約を作成します。[`LifecycleContext`] と [`AsyncLifecycleContext`] は完了前に保存とイベント配信の
    //! 成功を要求し、失敗は [`LifecycleCompletionError`] と [`LifecycleOperationError`] で報告します。
    //! [`LifecycleCallback`] と [`AsyncLifecycleCallback`] はコールバックを受け渡します。
    //! 失敗した操作は消費済みとなり、同じコンテキストから再試行できません。

    pub use crate::cqrs_impl::{
        AggregateCreator, AsyncAggregateCreator, AsyncCommandHandler, AsyncLifecycleCallback,
        AsyncLifecycleContext, AsyncQueryHandler, CommandHandler, LifecycleCallback,
        LifecycleCompletionError, LifecycleContext, LifecycleOperationError, QueryHandler,
    };
}

#[cfg(feature = "event_sourcing")]
pub mod event_sourcing {
    //! **English:**<br>
    //! Event-sourcing ports: event stores, replay, snapshots, and event versioning.
    //!
    //! This module is experimental: it may change in 0.x releases. It requires the `event_sourcing`
    //! feature, which is enabled by default.
    //!
    //! [`EventStore`] loads complete [`StreamHistory`] values and appends events using
    //! [`ExpectedRevision`]. [`AggregateReplay`] supplies reconstruction hooks for
    //! [`replay_aggregate`] and [`replay_with_optional_snapshot`]. [`SnapshotStore`]
    //! persists [`AggregateSnapshot`] values separately; adapters define storage and
    //! concurrency behavior. [`AsyncEventStore`] and [`AsyncSnapshotStore`] expose boxed
    //! `Send` futures for event and snapshot persistence. [`EventIdentity`] and
    //! [`VersionedEvent`] describe event schemas, while [`UpcasterChain`] applies
    //! [`EventUpcaster`] stages in insertion order without automatically selecting
    //! versions.
    //!
    //! **日本語:**<br>
    //! イベントストア、復元、スナップショット、イベントのバージョニングのポートを提供します。
    //!
    //! このモジュールは実験的で、0.x のリリースで変わる可能性があります。feature `event_sourcing`
    //! （既定で有効）が必要です。
    //!
    //! [`EventStore`] は完全な [`StreamHistory`] を読み込み、[`ExpectedRevision`] を条件に
    //! イベントを追加します。[`AggregateReplay`] は [`replay_aggregate`] と
    //! [`replay_with_optional_snapshot`] の復元フックを定めます。[`SnapshotStore`] は
    //! [`AggregateSnapshot`] を別途永続化し、保存や並行処理の挙動はアダプターが定めます。
    //! [`AsyncEventStore`] と [`AsyncSnapshotStore`] はイベントとスナップショットの永続化を、
    //! ボックス化された `Send` Future で公開します。[`EventIdentity`] と [`VersionedEvent`] は
    //! イベントのスキーマを表し、[`UpcasterChain`] はバージョンを自動選択せず、
    //! [`EventUpcaster`] の段階を追加順に適用します。

    pub use crate::async_event_store::AsyncEventStore;
    pub use crate::async_snapshot::AsyncSnapshotStore;
    pub use crate::event_store::{
        AppendError, AppendResult, CurrentRevision, EventStore, ExpectedRevision, LoadResult,
        NoStream, RevisionConflict, StreamHistory, StreamHistoryError, StreamRevision,
    };
    pub use crate::event_versioning::{
        EventIdentity, EventUpcaster, UpcasterChain, VersionedEvent,
    };
    pub use crate::replay::{
        AggregateReplay, RehydratedAggregate, ReplayError, SnapshotReplayError, replay_aggregate,
        replay_with_optional_snapshot,
    };
    pub use crate::snapshot::{
        AggregateSnapshot, SnapshotLoadError, SnapshotSaveError, SnapshotStore, SnapshotUse,
    };
}

#[cfg(feature = "delivery")]
pub mod delivery {
    //! **English:**<br>
    //! Reliable delivery ports: transactional outboxes, event consumers, and projections.
    //!
    //! This module is experimental: it may change in 0.x releases. It requires the `delivery`
    //! feature, which is enabled by default and enables `event_sourcing`.
    //!
    //! [`EventConsumer`] and [`Projection`] separate event handling from read-model updates, with
    //! asynchronous counterparts [`AsyncEventConsumer`] and [`AsyncProjection`].
    //! [`DomainEventDeliveryMetadata`] optionally exposes [`DeliveryMetadata`].
    //! [`TransactionalOutbox`] separates save-and-enqueue, claim, and acknowledgement operations;
    //! adapters supply transactions and delivery policies, not this crate.
    //!
    //! [`TransactionalOutboxLifecycleContext`] tracks a single save-and-enqueue attempt. It
    //! pulls pending events without restoring them on failure, rejects repeated attempts,
    //! and reports missing or unsuccessful enqueueing at completion. It does not implement
    //! a transaction.
    //!
    //! [`AsyncTransactionalOutbox`] provides future-based outbox operations.
    //! [`AsyncTransactionalOutboxLifecycleContext`] consumes an attempt and pulls events
    //! before polling; dropping its future does not restore events.
    //! [`ProjectionCheckpoint`] carries application-defined progress for
    //! [`ProjectionCheckpointStore`], while [`IdempotentProjection`] reports applied or
    //! duplicate input via [`ProjectionApplication`]. Implementations supply duplicate
    //! detection and coordination between checkpoint persistence and read-model changes.
    //!
    //! [`AsyncProjectionCheckpointStore`] persists progress through boxed `Send` futures,
    //! and [`AsyncIdempotentProjection`] asynchronously distinguishes new application from
    //! duplicates. These independent ports do not coordinate transactions; dropping a
    //! future does not guarantee rollback or absence of side effects. Implementations
    //! define cancellation and retry semantics.
    //!
    //! **日本語:**<br>
    //! 信頼性のある配送のポートとして、トランザクショナルアウトボックス、イベントコンシューマー、
    //! プロジェクションを提供します。
    //!
    //! このモジュールは実験的で、0.x のリリースで変わる可能性があります。feature `delivery`
    //! （既定で有効。`event_sourcing` を有効にします）が必要です。
    //!
    //! [`EventConsumer`] と [`Projection`] はイベント処理と読み取りモデル更新を分け、非同期版の
    //! [`AsyncEventConsumer`] と [`AsyncProjection`] も提供します。[`DomainEventDeliveryMetadata`] は
    //! 任意で [`DeliveryMetadata`] を公開します。[`TransactionalOutbox`] は保存・登録、取得、
    //! 確認応答を分離します。トランザクションと配信方針はこのクレートではなくアダプターが提供します。
    //!
    //! [`TransactionalOutboxLifecycleContext`] は保存・登録の一回の試行を追跡します。
    //! 保留イベントを取り出して失敗時も復元せず、繰り返しの試行を拒否し、完了確認時に
    //! 未試行や未成功を報告します。トランザクション自体は実装しません。
    //!
    //! [`AsyncTransactionalOutbox`] は Future によるアウトボックス操作を提供します。
    //! [`AsyncTransactionalOutboxLifecycleContext`] はポーリング前に試行を消費してイベントを取り出し、
    //! Future の破棄でも復元しません。[`ProjectionCheckpoint`] はアプリケーション定義の進捗を
    //! [`ProjectionCheckpointStore`] に渡し、[`IdempotentProjection`] は適用または重複を
    //! [`ProjectionApplication`] で報告します。重複検出と、チェックポイント永続化・読み取りモデル変更の
    //! 連携は実装が提供します。
    //!
    //! [`AsyncProjectionCheckpointStore`] はボックス化された `Send` Future で進捗を永続化し、
    //! [`AsyncIdempotentProjection`] は新規適用と重複を非同期に区別します。これらの独立したポートは
    //! トランザクションを調整せず、Future の破棄はロールバックや副作用の不在を保証しません。
    //! キャンセルと再試行の意味は実装が定めます。

    pub use crate::async_event_consumption::{AsyncEventConsumer, AsyncProjection};
    pub use crate::async_outbox_lifecycle::{
        AsyncOutboxLifecycleCompletionError, AsyncOutboxLifecycleContext,
        AsyncOutboxLifecycleOperationError, AsyncTransactionalOutboxCompletionError,
        AsyncTransactionalOutboxLifecycleContext, AsyncTransactionalOutboxOperationError,
    };
    pub use crate::async_transactional_outbox::AsyncTransactionalOutbox;
    pub use crate::event_consumption::{EventConsumer, Projection};
    pub use crate::event_delivery_metadata::{DeliveryMetadata, DomainEventDeliveryMetadata};
    pub use crate::outbox_lifecycle::{
        OutboxLifecycleCompletionError, OutboxLifecycleContext, OutboxLifecycleOperationError,
        TransactionalOutboxCompletionError, TransactionalOutboxLifecycleContext,
        TransactionalOutboxOperationError,
    };
    pub use crate::projection_reliability::{
        AsyncIdempotentProjection, AsyncProjectionCheckpointStore, IdempotentProjection,
        ProjectionApplication, ProjectionCheckpoint, ProjectionCheckpointStore,
    };
    pub use crate::transactional_outbox::{OutboxClaim, OutboxMessageId, TransactionalOutbox};
}

#[cfg(feature = "process")]
pub mod process {
    //! **English:**<br>
    //! Process managers that return state transitions and effect descriptions.
    //!
    //! This module is experimental: it may change in 0.x releases. It requires the `process`
    //! feature, which is enabled by default.
    //!
    //! [`ProcessManager`] returns a [`Transition`] of state and effect descriptions for callers to
    //! store and execute.
    //!
    //! **日本語:**<br>
    //! 状態遷移と副作用の記述を返すプロセスマネージャーを提供します。
    //!
    //! このモジュールは実験的で、0.x のリリースで変わる可能性があります。feature `process`
    //! （既定で有効）が必要です。
    //!
    //! [`ProcessManager`] は状態と副作用の記述を [`Transition`] で返し、保存と実行は
    //! 呼び出し側が担います。

    pub use crate::process_manager::{ProcessManager, Transition};
}

#[cfg(feature = "secret")]
pub mod secret {
    //! **English:**<br>
    //! Secret values that redact formatted output and zeroize on drop.
    //!
    //! This module is experimental: it may change in 0.x releases. It requires the `secret`
    //! feature, which is enabled by default.
    //!
    //! [`Secret`] wraps a value implementing `zeroize::Zeroize`, redacts its formatted
    //! output, and zeroizes it on drop. Access to the unredacted value is explicit; extracting it
    //! transfers responsibility for zeroization to the caller. [`SecretMetadata`] is a zero-sized
    //! marker and does not itself protect data. The `serde` feature adds `Secret` deserialization;
    //! `serde_with` includes `serde` and adds the explicit `ExposeSecret` output adapter.
    //! `Secret` never implements `Serialize`; adapter output is plaintext, not encryption.
    //!
    //! **日本語:**<br>
    //! 書式出力を秘匿化し、破棄時にゼロ化するシークレット値を提供します。
    //!
    //! このモジュールは実験的で、0.x のリリースで変わる可能性があります。feature `secret`
    //! （既定で有効）が必要です。
    //!
    //! [`Secret`] は `zeroize::Zeroize` を実装する値を包み、書式出力を秘匿化し、
    //! 破棄時にゼロ化します。秘匿化していない値には明示的にアクセスし、値を取り出した後の
    //! ゼロ化は呼び出し側が管理します。[`SecretMetadata`] はサイズがゼロのマーカーであり、
    //! それ自体はデータを保護しません。`serde` feature は `Secret` の復元に対応し、
    //! `serde_with` は `serde` を含み、明示的な出力アダプター `ExposeSecret` を追加します。
    //! `Secret` 自体は `Serialize` を実装しません。アダプターの出力は暗号化されない平文です。

    #[cfg(feature = "serde_with")]
    pub use crate::expose_secret::ExposeSecret;
    pub use crate::secret_impl::Secret;
    pub use crate::secret_metadata::SecretMetadata;
}

#[doc(hidden)]
/// **English:**<br>
/// Internal access to the optional `serde` dependency for generated implementations.
///
/// **日本語:**<br>
/// 生成された実装から任意依存の `serde` にアクセスするための内部モジュールです。
pub mod __private {
    /// **English:**<br>
    /// Re-exports the optional `serde` crate for macro-generated code.
    ///
    /// **日本語:**<br>
    /// マクロが生成したコードで使うために、任意依存の `serde` クレートを再エクスポートします。
    #[cfg(feature = "serde")]
    pub use serde;
}
