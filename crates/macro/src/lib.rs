//! **English:**<br>
//! Derive value objects, identity-based entities, domain-event markers,
//! and aggregate event queues for `ddd-toolkit-core`.
//!
//! Add both `ddd-toolkit-macro` and `ddd-toolkit-core` to your dependencies.
//! Expansion resolves the core package by name, including renamed dependencies;
//! a facade dependency alone is not resolved. These macros generate implementations
//! at compile time, not runtime synchronization or persistence.
//!
//! Use [`ValueObject`] for field-based equality, [`Entity`] for identity-based
//! equality, [`DomainEvent`] for event types, and [`AggregateRoot`] for a draining
//! event queue. Do not separately derive traits that a macro already implements.
//!
//! **日本語:**<br>
//! `ddd-toolkit-core` 向けに、値オブジェクト、識別子で比較する
//! エンティティ、ドメインイベントのマーカー、集約のイベントキューを導出します。
//!
//! 依存関係には `ddd-toolkit-macro` と `ddd-toolkit-core` の両方が必要です。
//! コアのパッケージ名から参照先を解決するため依存の別名に対応しますが、
//! ファサードだけの依存は解決しません。実装をコンパイル時に生成するもので、
//! 実行時の同期や永続化は行いません。
//!
//! フィールドの値で比較する型には [`ValueObject`]、識別子で比較する型には
//! [`Entity`]、イベント型には [`DomainEvent`]、イベントを取り出して空にする
//! キューには [`AggregateRoot`] を使います。生成済みのトレイトを重複して導出しないでください。
//!
//! # Example / 使用例
//!
//! ```rust
//! use ddd_toolkit_macro::ValueObject;
//!
//! #[derive(ValueObject)]
//! struct Quantity(u32);
//!
//! let quantity = Quantity::new(3);
//! assert_eq!(quantity.as_inner(), &3);
//! assert_eq!(quantity.into_inner(), 3);
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod aggregate_root;
mod domain_event;
mod entity;
mod path;
mod serde;
mod value_object;

/// **English:**<br>
/// Implements `ddd_toolkit_core::ValueObject` and value-based
/// `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`, with corresponding field bounds.
///
/// Accepts nonempty named or tuple structs and nonempty enums, not unions or unit
/// structs. A single-field struct exposes that field as `Inner`, `as_inner`, and
/// `into_inner`; multi-field structs and enums use `Self` instead. Structs get
/// `new` with fields in declaration order. Multi-field structs also get borrowed
/// accessors named after fields, or `field_0`, `field_1`, etc. for tuple fields.
/// Enums get `new_<variant>`, `is_<variant>`, and `as_<variant>`, using the fully
/// lowercased variant name (not snake case). Variant accessors return `None` for
/// another variant, `Some(())` for unit variants, or a tuple of field references.
/// Lowercased variant names must be distinct; generated names must not collide
/// with other methods or field accessors.
///
/// `#[value_object(trait_only)]` generates only the domain trait implementation,
/// with the same `Inner` mapping but no inherent methods, constructors, standard
/// traits, or extra bounds. It preserves handwritten APIs and cannot be combined
/// with `validate`, `error`, or `serde`. It does not enforce invariants on public
/// fields, direct enum construction, or independently implemented deserialization.
///
/// `#[value_object(validate = path, error = ErrorType)]` replaces `new` with
/// `try_new` (or `try_new_<variant>`). Both options are required together. The
/// validator receives `&Inner` for a single-field struct and `&Self` otherwise,
/// returns `Result<(), ErrorType>`, and runs before the constructor returns the
/// value. Its error is propagated; its panics are not caught. Validated struct
/// fields cannot be unrestricted `pub`. Restricted visibility, direct construction
/// inside the defining module, and enum variants can still bypass validation.
///
/// `#[value_object(serde)]` additionally generates serialization through the core
/// crate's serde re-export; enable the core crate's `serde` feature. Single-field
/// structs serialize as their inner value, named structs as structs, tuple structs
/// as tuples, and enums as externally tagged variants by default. Supported
/// container/variant/field `#[serde(...)]` attributes are delegated to Serde derive;
/// unsupported attributes (including container conversions/default) are rejected.
/// Single-field tuple variants
/// use Serde's newtype format; other tuple variants use tuple format.
/// Deserialization runs validation when configured,
/// mapping its errors to `value object validation failed`. Serialization generation
/// is selected by this attribute, not by a feature on the macro crate.
/// Invalid shapes, duplicate or unknown options, and missing core dependencies
/// produce compile errors; field types must satisfy the generated trait bounds.
///
/// **日本語:**<br>
/// `ddd_toolkit_core::ValueObject` と値に基づく `Debug`、`Clone`、
/// `PartialEq`、`Eq`、`Hash` を、各フィールドに必要な境界付きで実装します。
///
/// 空でない名前付き・タプル構造体と列挙型に対応し、共用体やユニット構造体は拒否します。
/// 単一フィールド構造体の `Inner`、`as_inner`、`into_inner` はそのフィールドを扱い、
/// 複数フィールド構造体と列挙型では `Self` を扱います。構造体には宣言順の引数を取る
/// `new` を生成します。複数フィールドの場合は名前付きフィールド名、またはタプルの
/// `field_0`、`field_1` などの名前で借用アクセサーも生成します。
/// 列挙型にはバリアント名をすべて小文字にした（snake case ではない）接尾辞で
/// `new_<variant>`、`is_<variant>`、`as_<variant>` を生成します。
/// バリアントアクセサーは不一致なら `None`、ユニットなら `Some(())`、それ以外は
/// フィールド参照のタプルを返します。小文字化した名前は一意である必要があり、
/// 生成する名前を他のメソッドやフィールドアクセサーと衝突させないでください。
///
/// `#[value_object(trait_only)]` は同じ `Inner` 対応のドメイントレイト実装だけを
/// 生成し、固有メソッド、コンストラクター、標準トレイト、追加の境界は生成しません。
/// 手書きの API を維持でき、`validate`、`error`、`serde` とは併用できません。
/// 公開フィールド、列挙型の直接構築、別途実装された逆直列化の不変条件は保証しません。
///
/// `#[value_object(validate = path, error = ErrorType)]` は `new` を `try_new`
/// （列挙型では `try_new_<variant>`）に置き換えます。両オプションの同時指定が必要です。
/// 検証関数は単一フィールドなら `&Inner`、それ以外なら `&Self` を受け取り、
/// `Result<(), ErrorType>` を返します。値を返す前に検証し、エラーを伝播します。
/// 検証関数のパニックは捕捉しません。検証付き構造体のフィールドは無制限の `pub` に
/// できませんが、制限付き公開、定義モジュール内の直接構築、列挙型のバリアントからは
/// 検証を迂回できます。
///
/// `#[value_object(serde)]` はコアの serde 再公開を使う直列化実装も生成します。
/// コアの `serde` feature を有効にしてください。単一フィールドは内包値、名前付き
/// 構造体は構造体、タプル構造体はタプル、列挙型は外部タグ形式で直列化します。
/// これらは既定の形式です。対応するコンテナー・バリアント・フィールドの
/// `#[serde(...)]` 属性は Serde derive に委譲し、非対応の属性（コンテナーの
/// 変換属性や default など）は拒否します。
/// 単一フィールドのタプルバリアントは Serde の newtype 形式、その他はタプル形式です。
/// 逆直列化時も設定された検証を
/// 実行し、検証エラーを `value object validation failed` に変換します。
/// 直列化の生成はマクロクレートの feature ではなく、この属性で選択します。
/// 非対応の形状、重複・未知のオプション、コア依存の欠落はコンパイルエラーになります。
/// フィールド型は生成するトレイトの境界を満たす必要があります。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_macro::ValueObject;
///
/// #[derive(ValueObject)]
/// #[value_object(validate = positive, error = &'static str)]
/// struct Quantity(u32);
///
/// fn positive(value: &u32) -> Result<(), &'static str> {
///     if *value > 0 { Ok(()) } else { Err("zero quantity") }
/// }
///
/// assert_eq!(Quantity::try_new(0), Err("zero quantity"));
/// assert_eq!(Quantity::try_new(2).unwrap().into_inner(), 2);
///
/// #[derive(ValueObject)]
/// enum Status { Pending, Ready(u32) }
/// let status = Status::new_ready(7);
/// assert_eq!(status.as_ready(), Some((&7,)));
/// assert!(!status.is_pending());
/// ```
#[proc_macro_derive(ValueObject, attributes(value_object, serde))]
pub fn derive_value_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    value_object::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// **English:**<br>
/// Implements `ddd_toolkit_core::Entity`, a borrowed inherent `id()`
/// accessor, and `PartialEq`, `Eq`, and `Hash` using only the identifier field.
///
/// Requires a named-field struct with exactly one `#[entity(id)]` field. Other
/// fields do not participate in comparison or hashing; identifier types must
/// implement the corresponding traits. By default no constructor or validation
/// is generated. Non-struct inputs, misplaced flags, missing or multiple identifier
/// fields, unknown options, and missing core dependencies cause compile errors.
///
/// `#[entity(validate = path, error = ErrorType)]` generates `try_new` taking all
/// fields in declaration order, including the supplied ID. The validator receives
/// `&Self` (even for one field) and returns `Result<(), ErrorType>`; errors and
/// panics propagate. Unrestricted `pub` fields are rejected. Direct construction
/// in the defining module, restricted visibility, and handwritten mutations can
/// still bypass validation; mutation methods must preserve invariants themselves.
///
/// `#[entity(serde)]` generates `Serialize` and `Deserialize` using the core
/// crate's serde re-export; enable the core or facade `serde` feature. All fields
/// participate, in struct format by default even for an ID-only entity. With
/// validation configured, deserialization runs the same validator as `try_new`, mapping failures to
/// `entity validation failed` without exposing the validator error. These paths
/// only construct supplied state: they do not generate IDs or creation events.
/// Use separate DTOs when persistence or wire formats differ from the domain model.
/// Supported field/container `#[serde(...)]` attributes are delegated to Serde
/// derive. Unsupported attributes, including container conversions/default, are rejected.
///
/// `#[entity(trait_only)]` generates only the domain trait implementation. It
/// still requires `#[entity(id)]`, but adds no inherent `id()`, standard traits,
/// or extra bounds, so handwritten accessors and equality semantics are preserved.
/// It cannot be combined with `validate`, `error`, or `serde`.
///
/// **日本語:**<br>
/// `ddd_toolkit_core::Entity` と識別子を借用する固有の `id()`、
/// 識別子フィールドだけを使う `PartialEq`、`Eq`、`Hash` を実装します。
///
/// 名前付き構造体に `#[entity(id)]` フィールドがちょうど一つ必要です。
/// 他のフィールドは比較やハッシュに使いません。識別子型は対応するトレイトを
/// 実装する必要があります。既定ではコンストラクターや検証は生成しません。
/// 構造体以外、フラグの誤配置、識別子の欠落・複数指定、未知のオプション、
/// コア依存の欠落はコンパイルエラーになります。
///
/// `#[entity(validate = path, error = ErrorType)]` は ID を含む全フィールドを
/// 宣言順に受け取る `try_new` を生成します。検証関数は単一フィールドでも
/// `&Self` を受け取り、`Result<(), ErrorType>` を返します。エラーとパニックは
/// 伝播します。無制限の `pub` フィールドは禁止しますが、定義モジュール内の
/// 直接構築、制限付き公開、手書きの変更処理では検証を迂回できるため、
/// 状態変更メソッド自身でも不変条件を維持してください。
///
/// `#[entity(serde)]` はコアの serde 再公開を使って `Serialize` と `Deserialize`
/// を生成します。コアまたはファサードの `serde` feature が必要です。全フィールドを
/// 対象とし、既定では ID のみでも構造体形式です。検証付きの逆直列化は
/// `try_new` と同じ検証を実行し、
/// 検証エラーの内容を公開せず `entity validation failed` に変換します。
/// 指定された状態を構築するだけで、ID の生成や作成イベントの発生は行いません。
/// 永続化・通信形式がドメインモデルと異なる場合は DTO を分けてください。
/// 対応するフィールド・コンテナーの `#[serde(...)]` 属性は Serde derive に委譲し、
/// 非対応の属性（コンテナーの変換属性や default など）は拒否します。
///
/// `#[entity(trait_only)]` はドメイントレイト実装だけを生成します。
/// `#[entity(id)]` は引き続き必要ですが、固有の `id()`、標準トレイト、追加の
/// 境界は生成しないため、手書きのアクセサーと等価性の意味を維持できます。
/// `validate`、`error`、`serde` との併用はできません。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_macro::Entity;
///
/// #[derive(Entity)]
/// struct Order {
///     #[entity(id)]
///     id: u64,
///     total: u32,
/// }
/// let first = Order { id: 1, total: 10 };
/// let updated = Order { id: 1, total: 20 };
/// assert!(first == updated);
/// assert_eq!(updated.id(), &1);
///
/// #[derive(Entity)]
/// #[entity(validate = check, error = &'static str)]
/// struct Stock {
///     #[entity(id)]
///     id: u64,
///     reserved: u32,
///     total: u32,
/// }
/// fn check(stock: &Stock) -> Result<(), &'static str> {
///     if stock.reserved <= stock.total { Ok(()) } else { Err("over-reserved") }
/// }
/// assert!(Stock::try_new(1, 5, 3).is_err());
/// assert_eq!(Stock::try_new(1, 2, 3).unwrap().id(), &1);
/// ```
#[proc_macro_derive(Entity, attributes(entity, serde))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    entity::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// **English:**<br>
/// Implements the empty `ddd_toolkit_core::DomainEvent` marker for
/// a struct or enum, preserving its generic parameters and where clause.
///
/// Does not add methods, serialization, `Send`, or `Sync` bounds. Event contents
/// and delivery remain the caller's responsibility. Unions and missing core
/// dependencies produce compile errors.
///
/// **日本語:**<br>
/// 構造体または列挙型に、空の `ddd_toolkit_core::DomainEvent`
/// マーカーを実装し、ジェネリック引数と where 節を維持します。
///
/// メソッド、直列化、`Send`・`Sync` 境界は追加しません。イベントの内容と配信は
/// 呼び出し側が管理します。共用体やコア依存の欠落はコンパイルエラーになります。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_macro::DomainEvent;
///
/// #[derive(DomainEvent)]
/// enum OrderEvent { Placed { order_id: u64 }, Cancelled }
/// fn accept_event(_: impl ddd_toolkit_core::DomainEvent) {}
/// accept_event(OrderEvent::Placed { order_id: 1 });
/// ```
#[proc_macro_derive(DomainEvent)]
pub fn derive_domain_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    domain_event::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// **English:**<br>
/// Implements `ddd_toolkit_core::AggregateRoot` using a struct's
/// `#[aggregate(events)]` vector as its pending event queue.
///
/// Requires a named-field struct that already implements `ddd_toolkit_core::Entity`
/// (for example via [`Entity`]) and exactly one marked field spelled `Vec<Event>`
/// or `std::vec::Vec<Event>`. Type aliases and other vector paths are not accepted;
/// `Event` must implement `ddd_toolkit_core::DomainEvent` (see [`DomainEvent`]).
/// `record_event` appends with `Vec::push`, preserving insertion order, and may
/// allocate. `pull_events` moves out the vector with `core::mem::take`, leaving an
/// empty queue; repeated pulls return empty vectors until another event is recorded.
/// These are trait methods, not inherent methods; import the core trait to call
/// them. Both require exclusive mutable access, with no locking, event publication,
/// persistence, or domain-state update generated. Vector growth has the usual
/// allocation and capacity limits of `Vec::push`.
/// Unsupported shapes, missing or duplicate queues, unknown or valued flags,
/// missing trait implementations, and missing core dependencies fail to compile.
///
/// **日本語:**<br>
/// 構造体の `#[aggregate(events)]` ベクターを未処理イベントキュー
/// として `ddd_toolkit_core::AggregateRoot` を実装します。
///
/// `ddd_toolkit_core::Entity` を実装済みの名前付き構造体（例: [`Entity`] による導出）
/// と、`Vec<Event>` または `std::vec::Vec<Event>` と記述されたキューフィールドが
/// ちょうど一つ必要です。型エイリアスや他のベクターパスは受け付けません。
/// `Event` は `ddd_toolkit_core::DomainEvent` を実装する必要があります（[`DomainEvent`]）。
/// `record_event` は `Vec::push` で挿入順に追加し、メモリを確保する場合があります。
/// `pull_events` は `core::mem::take` でベクターを取り出し、キューを空にします。
/// 次の記録まで再度取り出しても空のベクターを返します。固有メソッドではなくトレイト
/// メソッドなので、呼び出すにはコアのトレイトをインポートしてください。
/// どちらも排他的な可変アクセスを要求し、ロック、イベント配信、永続化、ドメイン状態の
/// 更新は生成しません。キューの拡張には `Vec::push` と同じメモリ確保・容量制限があります。
/// 非対応の形状、キューの欠落・重複、未知または値付きフラグ、必要なトレイト実装や
/// コア依存の欠落はコンパイルエラーになります。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_core::AggregateRoot as _;
/// use ddd_toolkit_macro::{AggregateRoot, DomainEvent, Entity};
///
/// #[derive(DomainEvent)]
/// struct Placed;
///
/// #[derive(Entity, AggregateRoot)]
/// struct Order {
///     #[entity(id)]
///     id: u64,
///     #[aggregate(events)]
///     events: Vec<Placed>,
/// }
/// let mut order = Order { id: 1, events: Vec::new() };
/// order.record_event(Placed);
/// assert_eq!(order.pull_events().len(), 1);
/// assert!(order.pull_events().is_empty());
/// ```
#[proc_macro_derive(AggregateRoot, attributes(aggregate))]
pub fn derive_aggregate_root(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    aggregate_root::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(test)]
mod expansion_tests;
