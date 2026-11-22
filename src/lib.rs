//! **English:**<br>
//! Re-exports the traits and helpers from `ddd-toolkit-core` 0.4: the central
//! [`Entity`], [`ValueObject`], [`AggregateRoot`], [`Repository`],
//! [`AsyncRepository`], and event publishing at the root, and the experimental
//! `cqrs`, `event_sourcing`, `delivery`, `process`, and `secret` modules behind
//! features of the same names (all on by default). Storage adapters and async runtimes
//! are supplied by the application.
//!
//! With the default `derive` feature, the `ValueObject`, `Entity`, `DomainEvent`,
//! and `AggregateRoot` derive macros are also available at this crate's root.
//! Generated paths prefer a direct `ddd-toolkit-core` dependency, then this
//! facade. Both dependencies may be renamed; a facade-only dependency suffices.
//!
//! The optional `serde` feature enables core's serde support and the serde
//! derives needed by generated deserializers. Serialization is opt-in per value
//! object with `#[value_object(serde)]`; put this flag last when combining options.
//! Single-field tuple enum variants use Serde's newtype format: `Code(7)` emits
//! `{"Code":7}`. Macro 0.5 changes the old `{"Code":[7]}` output; migrate saved
//! data by removing its outer tuple wrapper for affected variants before loading.
//! `secret::Secret` (feature `secret`) redacts formatting and zeroizes its contents
//! on drop. `serde` adds `Secret` deserialization without
//! implementing `Serialize`; `serde_with` includes `serde` and adds the explicit
//! `ExposeSecret` output adapter. Output is plaintext, not encryption; see `Secret`
//! for error-redaction and zeroization boundaries.
//!
//! Validation returns an application-defined error; construction
//! uses `try_new` rather than the removed `TryFrom` implementation.
//!
//! New to the crate? Follow the staged
//! [getting-started guide](https://github.com/ryoshrimp/ddd-toolkit/blob/main/docs/guide.md);
//! the API map below lists the public items by area; items in feature-gated modules
//! are shown as code and need the feature of the same name. For per-API examples and contract tests, see the
//! [public contract map](https://github.com/ryoshrimp/ddd-toolkit/blob/main/docs/public-contracts.md).
//! The map's areas roughly follow the guide's stages. The four derive macros (`ValueObject`,
//! `Entity`, `DomainEvent`, `AggregateRoot`; feature `derive`) share names with
//! the traits listed there, and `ExposeSecret` needs feature `serde_with`.
//!
//! **日本語:**<br>
//! `ddd-toolkit-core` 0.4 のトレイトとヘルパーを再公開します。ルートには中核の
//! [`Entity`]、[`ValueObject`]、[`AggregateRoot`]、[`Repository`]、
//! [`AsyncRepository`]、イベント発行を置き、実験的な `cqrs`、`event_sourcing`、
//! `delivery`、`process`、`secret` モジュールは同名の feature（既定で有効）で提供します。
//! ストレージのアダプターと非同期ランタイムはアプリケーションが提供します。
//!
//! 既定の `derive` feature では `ValueObject`、`Entity`、`DomainEvent`、
//! `AggregateRoot` の derive マクロもルートから利用できます。生成パスは直接依存の
//! `ddd-toolkit-core` を優先し、なければファサードを使います。両者の依存名変更に
//! 対応しており、ファサードだけの依存でも利用できます。
//!
//! 任意の `serde` feature はコアの serde 対応と生成された逆直列化に必要な serde の
//! derive を有効にします。直列化は型ごとに `#[value_object(serde)]` で指定し、
//! 他のオプションと併用するときはこのフラグを最後に置きます。単一フィールドの
//! タプル型 enum variant は newtype 形式で、`Code(7)` は `{"Code":7}` を出力します。
//! マクロ 0.5 では旧出力 `{"Code":[7]}` から変わるため、保存済みの対象バリアントの
//! 外側のタプルラッパーを除去してから読み込んでください。全 derive に直列化が
//! 自動適用されるわけではありません。
//! `secret::Secret`（feature `secret`）は書式出力を秘匿化して破棄時に内容をゼロ化します。
//! `serde` では `Serialize` を実装せずに復元に対応します。`serde_with` は `serde` を含み、
//! 明示的な出力用の `ExposeSecret` を追加します。出力は暗号化されない平文です。
//! エラー秘匿とゼロ化の範囲は `Secret` の説明を参照してください。
//!
//! 検証はアプリケーション定義のエラーを返します。廃止された
//! `TryFrom` 実装の代わりに `try_new` で構築します。
//!
//! 初めての方は、段階的な
//! [入門ガイド](https://github.com/ryoshrimp/ddd-toolkit/blob/main/docs/guide.ja.md)から
//! 始めてください。下の API マップは公開項目を領域別に一覧し、feature ゲートされたものは
//! コードで示します（feature ゲートされたモジュールの項目は同名の feature が必要）。API ごとの実行例と
//! 契約テストは[公開契約の対応表](https://github.com/ryoshrimp/ddd-toolkit/blob/main/docs/public-contracts.md)を
//! 参照してください。マップの領域はガイドのステージにおおむね対応します。四つの derive
//! マクロ（`ValueObject`、`Entity`、`DomainEvent`、`AggregateRoot`。feature `derive`）は
//! マップに挙げるトレイトと同名で、`ExposeSecret` には feature `serde_with` が必要です。
//!
//! # API map / API マップ
//!
//! ## Domain modeling / ドメインモデル
//!
//! [`ValueObject`], [`Entity`], [`AggregateRoot`], [`DomainEvent`],
//! [`DomainEventMetadata`], [`DomainEventConversion`]
//!
//! ## Persistence and publication / 永続化と発行
//!
//! [`Repository`], [`AsyncRepository`], [`DomainEventPublisher`],
//! [`AsyncDomainEventPublisher`], [`dispatch_events`], [`dispatch_events_async`]
//!
//! ## Experimental modules / 実験的モジュール
//!
//! - `cqrs` (feature `cqrs`): `CommandHandler`, `AsyncCommandHandler`, `QueryHandler`,
//!   `AsyncQueryHandler`, `AggregateCreator`, `AsyncAggregateCreator`, `LifecycleContext`,
//!   `AsyncLifecycleContext`, `LifecycleCallback`, `AsyncLifecycleCallback`,
//!   `LifecycleCompletionError`, `LifecycleOperationError`
//! - `event_sourcing` (feature `event_sourcing`): `EventStore`, `AsyncEventStore`,
//!   `ExpectedRevision`, `CurrentRevision`, `StreamRevision`, `NoStream`, `AppendResult`,
//!   `AppendError`, `RevisionConflict`, `LoadResult`, `StreamHistory`, `StreamHistoryError`,
//!   `AggregateReplay`, `RehydratedAggregate`, `ReplayError`, `SnapshotReplayError`,
//!   `replay_aggregate`, `replay_with_optional_snapshot`, `SnapshotStore`,
//!   `AsyncSnapshotStore`, `AggregateSnapshot`, `SnapshotUse`, `SnapshotLoadError`,
//!   `SnapshotSaveError`, `EventIdentity`, `VersionedEvent`, `EventUpcaster`, `UpcasterChain`
//! - `delivery` (feature `delivery`, includes `event_sourcing`): `TransactionalOutbox`,
//!   `AsyncTransactionalOutbox`, `OutboxClaim`, `OutboxMessageId`, the outbox lifecycle
//!   contexts and errors, `EventConsumer`, `AsyncEventConsumer`, `Projection`,
//!   `AsyncProjection`, `DeliveryMetadata`, `DomainEventDeliveryMetadata`,
//!   `IdempotentProjection`, `AsyncIdempotentProjection`, `ProjectionApplication`,
//!   `ProjectionCheckpoint`, `ProjectionCheckpointStore`, `AsyncProjectionCheckpointStore`
//! - `process` (feature `process`): `ProcessManager`, `Transition`
//! - `secret` (feature `secret`): `Secret`, `SecretMetadata`, and `ExposeSecret`
//!   (also needs `serde_with`)
//!
//! ## Other building blocks / その他の構成要素
//!
//! [`Specification`], [`And`], [`Or`], [`Not`], [`Policy`], [`DomainService`],
//! [`DomainFactory`], [`UnitOfWork`]
//!
//! # Example / 使用例
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! #[derive(ddd_toolkit::ValueObject)]
//! #[value_object(validate = validate_email, error = &'static str)]
//! struct Email(String);
//!
//! fn validate_email(value: &Email) -> Result<(), &'static str> {
//!     if value.0.contains('@') { Ok(()) } else { Err("missing @") }
//! }
//!
//! let email = Email::try_new("a@b.com".to_owned()).unwrap();
//! assert_eq!(email.as_inner(), "a@b.com");
//! assert_eq!(Email::try_new("invalid".to_owned()), Err("missing @"));
//! # }
//! # #[cfg(feature = "secret")] {
//! let key = ddd_toolkit::secret::Secret::new("sekret".to_owned());
//! assert_eq!(format!("{key:?}"), "[REDACTED]");
//! assert_eq!(key.expose_secret(|value| value.len()), 6);
//! # }
//! ```
#![forbid(unsafe_code)]

extern crate self as ddd_toolkit;

pub use ddd_toolkit_core::*;

#[cfg(feature = "derive")]
pub use ddd_toolkit_macro::*;
