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

既定モードでは、マクロが生成するトレイトを重複して実装・derive しないでください。
フィールドの型は、生成するトレイトに必要な境界を満たす必要があります。

### trait_only モード

既存 API やトレイトの実装方針を維持するには `#[value_object(trait_only)]`、
または構造体レベルの `#[entity(trait_only)]` を指定します。

```rust
use ddd_toolkit::{Entity, ValueObject};

#[derive(ValueObject)]
#[value_object(trait_only)]
struct Email(String);

impl Email {
    fn try_new(value: String) -> Result<Self, &'static str> {
        if value.contains('@') { Ok(Self(value)) } else { Err("missing @") }
    }
    fn as_inner(&self) -> &str { &self.0 }
}

#[derive(PartialEq, Eq, Entity)]
#[entity(trait_only)]
struct Account {
    #[entity(id)]
    id: u64,
    state: u32,
}

impl Account {
    fn id(&self) -> u64 { self.id }
}
```

- `ValueObject` はドメイントレイトの実装（`as_inner`／`into_inner`）だけを生成します。
  `Inner` の型は既定モードと同じです。`new`／`try_new`／バリアントのコンストラクター、
  固有メソッド、標準トレイト、serde 実装は生成しません。
- `Entity` はドメイントレイトの実装（`Id`／`id`）だけを生成します。
  `#[entity(id)]` フィールドは引き続きちょうど一つ必要です。固有の `id()` や
  ID 比較の `PartialEq`／`Eq`／`Hash` は生成しません。この例では構造比較を維持し、
  同じ ID でも状態が異なれば非等価になります。既存固有メソッドは異なるシグネチャでも
  共存でき、`Entity::id(&account)` でトレイトのメソッドを明示的に呼べます。
- 既存コンストラクター、検証、getter、秘匿用 `Debug`、`Clone` を実装しない方針などを
  維持できます。フィールドに新しいトレイト境界は追加しません。
  オプション未指定時の生成 API・意味は不変です。個別メソッドの生成選択は非対応です。
- `validate`／`error`／`serde` との併用は無視せずエラーにします。
  検証とシリアライズは利用側で実装してください。

これは生成コードの制限であり、公開フィールド、列挙型の直接構築、利用側の
デシリアライズなどを含めた不変条件の保護を保証しません。`Inner`、識別子の対応付け、
借用・所有権変換の動作自体を変更したい場合はドメイントレイトを手動実装してください。

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

### core の互換性

macro 0.5.x は core >=0.4.1, <0.5.0 に対応します。組み合わせを揃えるには現行の
facade を推奨します。検証版・依存名変更・生成コード API の安定性は
[互換性契約](../../docs/macro-compatibility.md) を参照してください。

### シリアライズ

**コア**の依存で serde 対応を有効にしてください。

```sh
cargo add ddd-toolkit-core --features serde
```

ファサードのみを使う場合は、代わりに `cargo add ddd-toolkit --features serde` を実行します。
rustc が `__private` 内の `serde` を `serde` feature で無効化されていると報告したら、
直接利用では **ddd-toolkit-core**、ファサード経由では **ddd-toolkit** の feature を有効に
してください。マクロクレートの feature ではありません。両方に依存する場合は直接の core
依存が優先されます。`#[value_object(serde)]` なしの通常の derive に serde は不要です。
独立 consumer テストで両構成の成功と feature 未有効時の診断を検証します。

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
ValueObject／Entity ともに対応する `#[serde(...)]` 属性は Serde derive に委譲し、
復元時の検証を維持します。例・対応属性・明示的な制限は
[ドメイン Serde ガイド](../../docs/macro-serde.md)を参照してください。
自分のコードで serde やフォーマット用クレートの API を使う場合は、それらへの直接依存を
追加してください。同じ `#[value_object(...)]` 属性で `validate`、`error` と併用できます。

単一フィールド構造体は内包値、複数フィールドの名前付き構造体は構造体、複数フィールドの
タプル構造体はタプル、列挙型は外部タグ形式でシリアライズします。
単一フィールドのタプルバリアントは Serde の newtype 形式です（`Code(7)` →
`{"Code":7}`）。その他はタプル形式を維持します。コンテナーのフィールドは自身の
形状を保ちます（`Codes(vec![7, 8])` → `{"Codes":[7,8]}`）。

**マクロ 0.5.0 / ファサード 0.4.0 の破壊的変更:** 旧シリアライザーは余分なタプルの
ラッパーを付け、`{"Code":[7]}` や `{"Codes":[[7,8]]}` を出力していました。
読み込み側の更新前に、保存レコードの対象バリアントから外側の配列を1段だけ除去して
ください。すべての配列を変換せず、スキーマやバージョン情報で旧レコードを識別します。
コンテナーを内包する場合は新旧の形式が曖昧になるため、自動フォールバックはありません。
送受信側の更新を調整するか、移行準備ができるまでマクロ 0.4 / ファサード 0.3 に
固定してください。他の Serde フォーマットも tuple から newtype に変わるため、
各フォーマットに応じた移行が必要です。

デシリアライズ時は設定された検証関数を実行し、検証エラーは元のメッセージではなく
`value object validation failed` に変換します。

## エンティティと集約イベント

`Entity` は `#[entity(id)]` フィールドだけで比較とハッシュを行い、他のフィールドは
使いません。識別子を借用する固有メソッド `id()` を生成します。
既定ではコンストラクターや検証は生成しません。

`#[entity(getters)]` を指定すると、フィールド名で `&T` を返す公開 getter を生成します:

```rust
#[derive(ddd_toolkit::Entity)]
#[entity(getters)]
struct Account {
    #[entity(id)]
    key: u64,
    name: String,
    #[entity(skip_getter)]
    secret: String,
}
// id() -> &u64 と name() -> &String を生成し、key() と secret() は生成しません。
```

`#[entity(skip_getter)]` は `getters` の指定が必要で、対象フィールドを除外します。
手書き getter との名前衝突を避ける場合にも使えます。識別子には `skip_getter` を
指定しても従来の `id()` を生成し、フィールド名での追加 getter は生成しません。
識別子以外に `id` という名前のフィールドがある場合は衝突を避けるため除外してください。
`getters` の指定がなければ従来どおりです。`trait_only` との併用はできませんが、
検証や serde とは併用できます。getter の除外は Serde の対象を変更しません。
また、参照を返しても内部可変性を持つ型の状態変更は防げません。

検証と serde はそれぞれ独立して、または組み合わせて指定できます:

```rust
use ddd_toolkit::Entity;

#[derive(Entity)]
#[entity(validate = check_stock, error = &'static str, serde)]
struct Stock {
    #[entity(id)]
    id: u64,
    reserved: u32,
    total: u32,
}

fn check_stock(stock: &Stock) -> Result<(), &'static str> {
    if stock.reserved <= stock.total { Ok(()) } else { Err("over-reserved") }
}

let stock = Stock::try_new(42, 2, 5).unwrap();
assert_eq!(stock.id(), &42);
assert!(Stock::try_new(42, 6, 5).is_err());
```

- `#[entity(serde)]` にはファサードまたはコアの `serde` feature が必要です。
  生成コードのために serde へ直接依存する必要はありません。ID のみでも全フィールドを
  既定では構造体形式で直列化します。対応するフィールド・コンテナーの `#[serde(...)]`
  属性は Serde derive に委譲します。対応範囲と制約は[ドメイン Serde ガイド](../../docs/macro-serde.md)を参照してください。
  通信・保存形式がドメイン構造と異なる場合は DTO を使ってください。
- `validate` と `error` は同時に指定します。単一フィールドでも検証関数は `&Self` を
  受け取ります。`try_new` は ID を含む全フィールドを宣言順に取り、検証のエラーと
  パニックを伝播します。検証なしではコンストラクターを生成せず、serde のみの復元は
  指定された状態をそのまま構築します。
- 検証付きの逆直列化は `try_new` と同じ検証を実行します。エラー型に `Display` を要求せず、
  失敗を `entity validation failed` に変換します。構築・復元のどちらも ID の生成、
  作成イベントの記録、手書きの業務上の作成メソッドの呼び出しは行いません。
- 検証付き Entity は無制限の `pub` フィールドを禁止しますが、定義モジュール内の
  直接構築、制限付き公開、手書きの変更処理では検証を迂回できます。状態変更メソッド
  自身でも不変条件を維持してください。構築時の検証は認可、DB の一意性検査、
  状態遷移のルールを代替しません。
- `trait_only` と `validate`、`error`、`serde` の併用は禁止です。手書きの `try_new`
  や serde 実装は生成コードと衝突しないようにしてください。

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
