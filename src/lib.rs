//! **English:**<br>
//! Re-exports the traits and helpers from `ddd-toolkit-core` 0.4,
//! including [`Entity`], [`ValueObject`], [`AggregateRoot`], [`Repository`],
//! [`AsyncRepository`], event publishing, CQRS, and event-sourcing contracts.
//! Storage adapters and async runtimes are supplied by the application.
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
//! [`Secret`] requires no feature and redacts formatting and zeroizes its contents
//! on drop without a feature flag.
//!
//! Validation returns an application-defined error; construction
//! uses `try_new` rather than the removed `TryFrom` implementation.
//!
//! **日本語:**<br>
//! `ddd-toolkit-core` 0.4 のトレイトとヘルパーを再公開します。
//! [`Entity`]、[`ValueObject`]、[`AggregateRoot`]、[`Repository`]、
//! [`AsyncRepository`]、イベント発行、CQRS、イベントソーシングの契約を含みます。
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
//! [`Secret`] は feature 指定なしで使え、書式出力を秘匿化して破棄時に内容をゼロ化します。
//!
//! 検証はアプリケーション定義のエラーを返します。廃止された
//! `TryFrom` 実装の代わりに `try_new` で構築します。
//!
//! # Example / 使用例
//!
//! ```rust
//! # #[cfg(feature = "derive")] {
//! #[derive(ddd_toolkit::ValueObject)]
//! #[value_object(validate = validate_email, error = &'static str)]
//! struct Email(String);
//!
//! fn validate_email(value: &str) -> Result<(), &'static str> {
//!     if value.contains('@') { Ok(()) } else { Err("missing @") }
//! }
//!
//! let email = Email::try_new("a@b.com".to_owned()).unwrap();
//! assert_eq!(email.as_inner(), "a@b.com");
//! assert_eq!(Email::try_new("invalid".to_owned()), Err("missing @"));
//! # }
//! let key = ddd_toolkit::Secret::new("sekret".to_owned());
//! assert_eq!(format!("{key:?}"), "[REDACTED]");
//! assert_eq!(key.expose_secret(|value| value.len()), 6);
//! ```

extern crate self as ddd_toolkit;

pub use ddd_toolkit_core::*;

#[cfg(feature = "derive")]
pub use ddd_toolkit_macro::*;
