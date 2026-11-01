//! **English:**<br>
//! Core traits for modeling Domain-Driven Design entities and value objects.
//!
//! [`Entity`] exposes a stable identifier by reference, while [`ValueObject`] exposes
//! a wrapped value by reference or by value. Implement these traits on domain types to keep their
//! domain-facing APIs independent from persistence details.
//!
//! For event-sourced aggregates, [`EventStore`] loads complete [`StreamHistory`] values
//! and appends events using [`ExpectedRevision`]. [`AggregateReplay`] supplies reconstruction hooks
//! for [`replay_aggregate`] and [`replay_with_optional_snapshot`]. [`SnapshotStore`] persists
//! [`AggregateSnapshot`] values separately; adapters define storage and concurrency behavior.
//!
//! [`Secret`] wraps a value implementing `zeroize::Zeroize`, redacts its formatted
//! output, and zeroizes it on drop. Access to the unredacted value is explicit; extracting it
//! transfers responsibility for zeroization to the caller. [`SecretMetadata`] is a zero-sized
//! marker and does not itself protect data.
//!
//! [`AsyncEventStore`] and [`AsyncSnapshotStore`] expose boxed `Send` futures for
//! event and snapshot persistence. [`EventConsumer`] and [`Projection`] separate event handling
//! from read-model updates, with asynchronous counterparts [`AsyncEventConsumer`] and
//! [`AsyncProjection`]. [`DomainEventDeliveryMetadata`] optionally exposes [`DeliveryMetadata`].
//! [`TransactionalOutbox`] separates save-and-enqueue, claim, and acknowledgement operations;
//! adapters supply transactions and delivery policies, not this crate.
//!
//! [`TransactionalOutboxLifecycleContext`] tracks a single save-and-enqueue attempt.
//! It pulls pending events without restoring them on failure, rejects repeated attempts, and
//! reports missing or unsuccessful enqueueing at completion. It does not implement a transaction.
//!
//! [`AsyncTransactionalOutbox`] provides future-based outbox operations.
//! [`AsyncTransactionalOutboxLifecycleContext`] consumes an attempt and pulls events before polling;
//! dropping its future does not restore events. [`ProjectionCheckpoint`] carries application-defined
//! progress for [`ProjectionCheckpointStore`], while [`IdempotentProjection`] reports applied or
//! duplicate input via [`ProjectionApplication`]. Implementations supply duplicate detection and
//! coordination between checkpoint persistence and read-model changes.
//!
//! [`AsyncProjectionCheckpointStore`] persists progress through boxed `Send` futures,
//! and [`AsyncIdempotentProjection`] asynchronously distinguishes new application from duplicates.
//! These independent ports do not coordinate transactions; dropping a future does not guarantee
//! rollback or absence of side effects. Implementations define cancellation and retry semantics.
//!
//! [`Specification`] composes fallible predicates with short-circuiting [`And`],
//! [`Or`], and [`Not`]. [`DomainService`], [`Policy`], and [`DomainFactory`] define typed operation,
//! rule-checking, and construction boundaries without prescribing persistence or thread safety.
//! [`ProcessManager`] returns a [`Transition`] of state and effect descriptions for callers to
//! store and execute. [`UnitOfWork`] exposes explicit registration, commit, and rollback ports;
//! implementations define their transaction guarantees. [`EventIdentity`] and [`VersionedEvent`]
//! describe event schemas, while [`UpcasterChain`] applies [`EventUpcaster`] stages in insertion
//! order without automatically selecting versions.
//!
//! **日本語:**<br>
//! ドメイン駆動設計のエンティティと値オブジェクトを表現するためのコアトレイトを提供します。
//!
//! [`Entity`] は安定した識別子を参照で公開し、[`ValueObject`] は内包値を参照または
//! 値で公開します。永続化の詳細から独立したドメイン型の API を保つために、これらのトレイトを実装します。
//!
//! イベントソーシングを使う集約では、[`EventStore`] が完全な [`StreamHistory`] を
//! 読み込み、[`ExpectedRevision`] を条件にイベントを追加します。[`AggregateReplay`] は
//! [`replay_aggregate`] と [`replay_with_optional_snapshot`] の復元フックを定めます。
//! [`SnapshotStore`] は [`AggregateSnapshot`] を別途永続化し、保存や並行処理の挙動は
//! アダプターが定めます。
//!
//! [`Secret`] は `zeroize::Zeroize` を実装する値を包み、書式出力を秘匿化し、
//! 破棄時にゼロ化します。秘匿化していない値には明示的にアクセスし、値を取り出した後の
//! ゼロ化は呼び出し側が管理します。[`SecretMetadata`] はサイズがゼロのマーカーであり、
//! それ自体はデータを保護しません。
//!
//! [`AsyncEventStore`] と [`AsyncSnapshotStore`] はイベントとスナップショットの
//! 永続化を、ボックス化された `Send` Future で公開します。[`EventConsumer`] と [`Projection`]
//! はイベント処理と読み取りモデル更新を分け、非同期版の [`AsyncEventConsumer`] と
//! [`AsyncProjection`] も提供します。[`DomainEventDeliveryMetadata`] は任意で
//! [`DeliveryMetadata`] を公開します。[`TransactionalOutbox`] は保存・登録、取得、確認応答を
//! 分離します。トランザクションと配信方針はこのクレートではなくアダプターが提供します。
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
//!
//! [`Specification`] は失敗し得る述語を、短絡評価する [`And`]、[`Or`]、[`Not`] で
//! 結合します。[`DomainService`]、[`Policy`]、[`DomainFactory`] は永続化やスレッド安全性を
//! 規定せず、型付きの操作、規則確認、構築の境界を定めます。[`ProcessManager`] は状態と
//! 副作用の記述を [`Transition`] で返し、保存と実行は呼び出し側が担います。[`UnitOfWork`] は
//! 明示的な登録、コミット、ロールバックのポートであり、トランザクションの保証は実装が定めます。
//! [`EventIdentity`] と [`VersionedEvent`] はイベントのスキーマを表し、[`UpcasterChain`] は
//! バージョンを自動選択せず、[`EventUpcaster`] の段階を追加順に適用します。

// Generated derives use the library name in both library and consumer targets.
extern crate self as ddd_toolkit_core;

mod aggregate_root;
mod async_dispatch;
mod async_domain_event_publisher;
mod async_event_consumption;
mod async_event_store;
mod async_outbox_lifecycle;
mod async_repository;
mod async_snapshot;
mod async_transactional_outbox;
mod cqrs;
mod dispatch;
mod domain_event;
mod domain_event_conversion;
mod domain_event_metadata;
mod domain_event_publisher;
mod domain_factory;
mod domain_service;
mod entity;
mod event_consumption;
mod event_delivery_metadata;
mod event_store;
mod event_versioning;
mod outbox_lifecycle;
mod process_manager;
mod projection_reliability;
mod replay;
mod repository;
mod secret;
mod secret_metadata;
mod snapshot;
mod specification;
mod transactional_outbox;
mod unit_of_work;
mod value_object;

pub use aggregate_root::AggregateRoot;

pub use async_dispatch::dispatch_events_async;

pub use async_domain_event_publisher::AsyncDomainEventPublisher;

pub use async_event_consumption::{AsyncEventConsumer, AsyncProjection};

pub use async_event_store::AsyncEventStore;

pub use async_outbox_lifecycle::{
    AsyncOutboxLifecycleCompletionError, AsyncOutboxLifecycleContext,
    AsyncOutboxLifecycleOperationError, AsyncTransactionalOutboxCompletionError,
    AsyncTransactionalOutboxLifecycleContext, AsyncTransactionalOutboxOperationError,
};

pub use async_repository::AsyncRepository;

pub use async_snapshot::AsyncSnapshotStore;

pub use async_transactional_outbox::AsyncTransactionalOutbox;

pub use cqrs::{
    AggregateCreator, AsyncAggregateCreator, AsyncCommandHandler, AsyncLifecycleCallback,
    AsyncLifecycleContext, AsyncQueryHandler, CommandHandler, LifecycleCallback,
    LifecycleCompletionError, LifecycleContext, LifecycleOperationError, QueryHandler,
};

pub use dispatch::dispatch_events;

pub use domain_event::DomainEvent;

pub use domain_event_conversion::DomainEventConversion;

pub use domain_event_metadata::DomainEventMetadata;

pub use domain_event_publisher::DomainEventPublisher;

pub use domain_factory::DomainFactory;

pub use domain_service::{DomainService, Policy};

pub use entity::Entity;

pub use event_consumption::{EventConsumer, Projection};

pub use event_delivery_metadata::{DeliveryMetadata, DomainEventDeliveryMetadata};

pub use event_store::{
    AppendError, AppendResult, CurrentRevision, EventStore, ExpectedRevision, LoadResult, NoStream,
    RevisionConflict, StreamHistory, StreamHistoryError, StreamRevision,
};

pub use event_versioning::{EventIdentity, EventUpcaster, UpcasterChain, VersionedEvent};

pub use outbox_lifecycle::{
    OutboxLifecycleCompletionError, OutboxLifecycleContext, OutboxLifecycleOperationError,
    TransactionalOutboxCompletionError, TransactionalOutboxLifecycleContext,
    TransactionalOutboxOperationError,
};

pub use process_manager::{ProcessManager, Transition};

pub use projection_reliability::{
    AsyncIdempotentProjection, AsyncProjectionCheckpointStore, IdempotentProjection,
    ProjectionApplication, ProjectionCheckpoint, ProjectionCheckpointStore,
};

pub use replay::{
    AggregateReplay, RehydratedAggregate, ReplayError, SnapshotReplayError, replay_aggregate,
    replay_with_optional_snapshot,
};

pub use repository::Repository;

pub use secret::Secret;

pub use secret_metadata::SecretMetadata;

pub use snapshot::{
    AggregateSnapshot, SnapshotLoadError, SnapshotSaveError, SnapshotStore, SnapshotUse,
};

pub use specification::{And, Not, Or, Specification};

pub use transactional_outbox::{OutboxClaim, OutboxMessageId, TransactionalOutbox};

pub use unit_of_work::UnitOfWork;

pub use value_object::ValueObject;

#[cfg(feature = "serde")]
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
    pub use serde;
}
