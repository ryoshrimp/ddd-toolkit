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
//! marker and does not itself protect data. The `serde` feature adds `Secret` deserialization;
//! `serde_with` includes `serde` and adds the explicit `ExposeSecret` output adapter.
//! `Secret` never implements `Serialize`; adapter output is plaintext, not encryption.
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
//! Every async port returns `Pin<Box<dyn Future + Send + 'a>>`. Implementations whose futures are
//! `!Send` (for example on WASM or single-threaded executors that hold `Rc` or `RefCell` across
//! `.await`) do not fit these signatures; there are no non-`Send` variants. Moving to native
//! `async fn` in traits is deferred until return type notation is stable, so that callers, not the
//! trait, can choose whether to require `Send`.
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
//! The thin traits ([`DomainEvent`], [`DomainService`], [`Policy`], [`DomainFactory`], and
//! [`Specification`]) add no behavior of their own, apart from the combinators ([`And`], [`Or`],
//! [`Not`]) that [`Specification`] provides. They help when code is generic over a role
//! (for example `P: Policy<Order>`) instead of a concrete type, when tests swap in a stub
//! implementation, and when signatures and docs should name the domain role. They are optional:
//! if a rule or operation is used in one place and never swapped, a plain function or inherent
//! method is enough. They provide no transaction, retry, persistence, or thread-safety behavior.
//! The example at the end of this page shows a [`Policy`] used as a bound and replaced by a stub.
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
//! それ自体はデータを保護しません。`serde` feature は `Secret` の復元に対応し、
//! `serde_with` は `serde` を含み、明示的な出力アダプター `ExposeSecret` を追加します。
//! `Secret` 自体は `Serialize` を実装しません。アダプターの出力は暗号化されない平文です。
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
//! 非同期ポートはすべて `Pin<Box<dyn Future + Send + 'a>>` を返します。Future が `!Send` になる実装
//! （たとえば WASM や、`.await` をまたいで `Rc` や `RefCell` を保持する単一スレッド executor 向けの実装）は
//! これらのシグネチャに適合せず、`Send` を要求しない版もありません。`Send` を要求するかどうかを trait ではなく
//! 呼び出し側が選べるように、ネイティブの `async fn` in trait への移行は return type notation の安定化まで見送ります。
//!
//! [`Specification`] は失敗し得る述語を、短絡評価する [`And`]、[`Or`]、[`Not`] で
//! 結合します。[`DomainService`]、[`Policy`]、[`DomainFactory`] は永続化やスレッド安全性を
//! 規定せず、型付きの操作、規則確認、構築の境界を定めます。[`ProcessManager`] は状態と
//! 副作用の記述を [`Transition`] で返し、保存と実行は呼び出し側が担います。[`UnitOfWork`] は
//! 明示的な登録、コミット、ロールバックのポートであり、トランザクションの保証は実装が定めます。
//! [`EventIdentity`] と [`VersionedEvent`] はイベントのスキーマを表し、[`UpcasterChain`] は
//! バージョンを自動選択せず、[`EventUpcaster`] の段階を追加順に適用します。
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
mod event_restoration_guard;
mod event_store;
mod event_versioning;
#[cfg(feature = "serde_with")]
mod expose_secret;
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

#[cfg(feature = "serde_with")]
pub use expose_secret::ExposeSecret;

pub use secret_metadata::SecretMetadata;

pub use snapshot::{
    AggregateSnapshot, SnapshotLoadError, SnapshotSaveError, SnapshotStore, SnapshotUse,
};

pub use specification::{And, Not, Or, Specification};

pub use transactional_outbox::{OutboxClaim, OutboxMessageId, TransactionalOutbox};

pub use unit_of_work::UnitOfWork;

pub use value_object::ValueObject;

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
