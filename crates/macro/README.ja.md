# ddd-toolkit-macro

[English](README.md) | 日本語

[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit-macro)](https://docs.rs/ddd-toolkit-macro)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/ddd-toolkit-macro.svg)](#ライセンス)

[`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit/tree/main/crates/core) 向けの
derive マクロです。値オブジェクト、識別子で比較するエンティティ、ドメインイベント、
集約のイベントキューを扱います。トレイト実装とアクセサーを生成するものであり、
永続化、イベント配信、同期機構は提供しません。

開発は [ddd-toolkit モノレポ](https://github.com/ryoshrimp/ddd-toolkit)で行います。
開発・リリース手順はルートリポジトリの CONTRIBUTING.md を参照してください。

## インストール

最新の Rust stable（edition 2024）で検証しています。
最低サポート Rust バージョン（MSRV）は定めていません。

```sh
cargo add ddd-toolkit-core ddd-toolkit-macro
```

生成パスは直接依存の `ddd-toolkit-core` を優先し、なければ `ddd-toolkit` の
再エクスポートを使います。両者の依存名変更に対応しています。ファサードだけへの
依存でも、既定の `derive` feature を使って利用できます。

## Derive

| Derive | 対応する入力 | 生成する API |
|---|---|---|
| `ValueObject` | 空でない名前付き・タプル構造体、空でない列挙型 | コアの `ValueObject`、`Debug`、`Clone`、`PartialEq`、`Eq`、`Hash`、コンストラクター、アクセサー |
| `Entity` | `#[entity(id)]` フィールドをちょうど一つ持つ名前付き構造体 | コアの `Entity`、固有メソッド `id()`、識別子に基づく `PartialEq`、`Eq`、`Hash` |
| `DomainEvent` | 構造体または列挙型 | コアの `DomainEvent` マーカー |
| `AggregateRoot` | コアの `Entity` を実装し、`#[aggregate(events)]` フィールドをちょうど一つ持つ名前付き構造体 | `record_event` と `pull_events` を備えたコアの `AggregateRoot` |

マクロが生成するトレイトを重複して実装・derive しないでください。
フィールドの型は、生成するトレイトに必要な境界を満たす必要があります。

## 値オブジェクト

単一フィールド構造体の `Inner` はそのフィールドの型です。`as_inner()` はフィールドを
借用し、`into_inner()` は値として返します。複数フィールド構造体と列挙型の `Inner` は
`Self` であり、これらのメソッドは値全体を借用または返却します。

構造体にはフィールドの宣言順に引数を取る `new(...)` を生成します。
複数フィールドの名前付き構造体にはフィールド名の借用アクセサーも生成し、
複数フィールドのタプル構造体には `field_0()`、`field_1()` などを生成します。

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct Quantity(u32);

#[derive(ValueObject)]
struct Address {
    city: String,
    postal_code: String,
}

#[derive(ValueObject)]
enum Status {
    Pending,
    Ready(u32),
}

fn main() {
    let quantity = Quantity::new(3);
    assert_eq!(quantity.as_inner(), &3);
    assert_eq!(quantity.into_inner(), 3);

    let address = Address::new("Tokyo".into(), "100-0001".into());
    assert_eq!(address.city(), "Tokyo");
    assert_eq!(address.as_inner(), &address);

    let status = Status::new_ready(7);
    assert!(status.is_ready());
    assert_eq!(status.as_ready(), Some((&7,)));
    assert_eq!(status.as_pending(), None);
    assert_eq!(Status::new_pending().as_pending(), Some(()));
}
```

列挙型には `new_<variant>`、`is_<variant>`、`as_<variant>` を生成します。
接尾辞は snake case ではなくバリアント名全体を小文字にした名前です。
たとえば `InProgress` は `inprogress` になります。バリアントのアクセサーは、不一致なら
`None`、ユニットバリアントなら `Some(())`、データを持つバリアントならフィールド参照の
タプルを返します。フィールドが一つでもタプルです。小文字化した名前は一意である必要があり、
生成するメソッド名を他のメソッドやフィールドのアクセサーと衝突させないでください。

### 検証

`validate` と `error` は同時に指定します。検証関数は引用符付きの文字列ではなく
Rust のパスで指定します。単一フィールド構造体では `&Inner`、複数フィールド構造体と
列挙型では `&Self` を受け取り、`Result<(), ErrorType>` を返します。

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(validate = validate_email, error = &'static str)]
struct Email(String);

fn validate_email(value: &String) -> Result<(), &'static str> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("missing @")
    }
}

fn main() {
    let email = Email::try_new("a@b.com".into()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(Email::try_new("invalid".into()), Err("missing @"));
}
```

検証を指定すると `new` は `try_new` に、`new_<variant>` は `try_new_<variant>` に
置き換わります。コンストラクターのエラーはそのまま伝播し、検証関数のパニックは捕捉しません。

検証付き構造体のフィールドを無制限の `pub` にすることはできません。ただし、すべての
迂回を防ぐものではありません。定義モジュール内の直接構築、制限付きのフィールド公開、
列挙型のバリアントの直接構築では検証を迂回できます。

### シリアライズ

**コア**の依存で serde 対応を有効にしてください。

```sh
cargo add ddd-toolkit-core --features serde
```

そのうえで、値オブジェクトごとに属性を指定します。

```rust
use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(serde)]
struct Quantity(u32);

fn main() {
    assert_eq!(Quantity::new(3).into_inner(), 3);
}
```

`#[value_object(serde)]` はコアの serde 再エクスポートを使って `Serialize` と
`Deserialize` を生成します。`ddd-toolkit-macro` 自体の feature ではありません。
自分のコードで serde やフォーマット用クレートの API を使う場合は、それらへの直接依存を
追加してください。同じ `#[value_object(...)]` 属性で `validate`、`error` と併用できます。

単一フィールド構造体は内包値、複数フィールドの名前付き構造体は構造体、複数フィールドの
タプル構造体はタプル、列挙型は外部タグ形式でシリアライズします。
タプルバリアントはフィールドが一つでもタプル形式です。デシリアライズ時は設定された
検証関数を実行し、検証エラーは元のメッセージではなく `value object validation failed` に変換します。

## エンティティと集約イベント

`Entity` は `#[entity(id)]` フィールドだけで比較とハッシュを行い、他のフィールドは
使いません。識別子を借用する固有メソッド `id()` を生成しますが、コンストラクターや
識別子の検証は生成しません。

`DomainEvent` は構造体または列挙型へのマーカー実装です。
シリアライズ、メソッド、`Send` / `Sync` 境界は追加しません。

```rust
use ddd_toolkit_core::AggregateRoot as _;
use ddd_toolkit_macro::{AggregateRoot, DomainEvent, Entity};

#[derive(DomainEvent, Debug, PartialEq)]
enum OrderEvent {
    Placed { order_id: u64 },
}

#[derive(Entity, AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    total: u32,
    #[aggregate(events)]
    events: Vec<OrderEvent>,
}

fn main() {
    let mut order = Order { id: 1, total: 10, events: Vec::new() };
    let updated = Order { id: 1, total: 20, events: Vec::new() };
    assert_ne!(order.total, updated.total);
    assert!(order == updated);
    assert_eq!(order.id(), &1);

    order.record_event(OrderEvent::Placed { order_id: 1 });
    assert_eq!(order.pull_events(), vec![OrderEvent::Placed { order_id: 1 }]);
    assert!(order.pull_events().is_empty());
}
```

イベントキューは `Vec<Event>` または `std::vec::Vec<Event>` と記述してください。
型エイリアスや他のベクターパスは受け付けません。イベント型にはコアの `DomainEvent`
の実装が必要です。

`record_event` と `pull_events` を呼び出すには、コアの `AggregateRoot` トレイトを
インポートしてください。固有メソッドではなくトレイトメソッドです。記録は挿入順に
キューへ追加し、メモリを確保する場合があります。取り出しはキューのイベントを移動し、
空のベクターを残します。どちらも可変アクセスが必要であり、イベントの発行、永続化、
ドメイン状態の更新は行いません。

## 属性

| 属性 | 指定箇所 | 効果 |
|---|---|---|
| `#[value_object(validate = path, error = ErrorType)]` | `ValueObject` の構造体または列挙型 | 検証付きの失敗し得るコンストラクターを生成 |
| `#[value_object(serde)]` | `ValueObject` の構造体または列挙型 | serde のシリアライズ・デシリアライズを生成。コアの `serde` feature が必要 |
| `#[entity(id)]` | 名前付き構造体の一つのフィールド | エンティティの識別子を選択 |
| `#[aggregate(events)]` | 名前付き構造体の一つのフィールド | 集約の保留イベントベクターを選択 |

詳細と実行可能な使用例は [API ドキュメント](https://docs.rs/ddd-toolkit-macro)を参照してください。

## ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) または [MIT license](LICENSE-MIT) の
いずれかを選択して利用できます。
