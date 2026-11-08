# ddd-toolkit

[English](README.md) | 日本語

[![crates.io](https://img.shields.io/crates/v/ddd-toolkit.svg)](https://crates.io/crates/ddd-toolkit)
[![docs.rs](https://img.shields.io/docsrs/ddd-toolkit)](https://docs.rs/ddd-toolkit)
[![CI](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ryoshrimp/ddd-toolkit/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/ddd-toolkit.svg)](#ライセンス)

ドメイン駆動設計（DDD）の契約と derive をアプリケーションに導入したい開発者向けの
Rust ツールキットです。同一性、値の検証、集約のイベントキュー、ドメインイベントを、
対応するトレイト実装の手書きを減らして表現できます。既存のストレージやランタイムの選択を
補完するものであり、永続化・トランザクション・配送保証を自動実装するフレームワークではありません。

**0.x・実験的:** API は変更される可能性があります。既存の
[互換性・破壊的変更の方針](docs/macro-compatibility.md#日本語)を参照してください。
API・ドキュメント・ユースケースへのフィードバックを
[GitHub Issues](https://github.com/ryoshrimp/ddd-toolkit/issues)で歓迎します。

[四つの derive とクイックスタート](#四つの-derive)から始め、
[注文の入門ガイド](examples/README.ja.md#初めての方へ)、必要に応じて
[Advanced（発展）](#advanced発展)へ進んでください。
[公開契約の対応表](docs/public-contracts.md#日本語)では、API 名から実行例・契約テストを探せ、
同期／非同期の区別と feature 条件も確認できます。

## パッケージとツールチェーン

ドメイン駆動設計のトレイトとヘルパーを提供するファサードクレートです。
[`ddd-toolkit-core`](crates/core) 0.4.6 の公開 API と、
[`ddd-toolkit-macro`](crates/macro) 0.5.5 の四つの derive マクロを再エクスポートします。
公開 API はすべて `ddd_toolkit::` から利用できます。

最新の Rust stable（edition 2024）で検証しています。
最低サポート Rust バージョン（MSRV）は定めていません。

このリポジトリは、ファサード・core・macro を含む Cargo ワークスペースです。
crates.io では別々のパッケージとして公開し、開発とテストを一緒に行います。
開発手順は [CONTRIBUTING.md](CONTRIBUTING.md)、公開手順は
[リリースガイド](docs/releasing.md)を参照してください。

## インストール

クイックスタートでは、デフォルトの feature（`derive` が有効）を使います:

```toml
[dependencies]
ddd-toolkit = "0.4"
```

トレイトとヘルパーだけを使い、derive マクロを使わない場合:

```toml
[dependencies]
ddd-toolkit = { version = "0.4", default-features = false }
```

ファサードだけへの依存で derive を利用できます。生成パスは直接依存の
`ddd-toolkit-core` があれば優先し、なければファサードの再エクスポートを使います。
コアとファサードのどちらの依存名変更にも対応しています。

ファサード 0.3.0 は core/macro 0.4.0 を採用しており、ファサード 0.2 から
破壊的な API 変更があります。後述の移行ガイドと [CHANGELOG.md](CHANGELOG.md)を参照してください。

## 四つの derive

| Derive | 最初に使う機能 |
| --- | --- |
| `ValueObject` | 等値比較と任意の検証を持つ値 |
| `Entity` | `#[entity(id)]` による同一性ベースの等値比較とハッシュ |
| `AggregateRoot` | `#[aggregate(events)]` のキューによる保留ドメインイベントの管理 |
| `DomainEvent` | イベントの契約を実装するドメインイベント型 |

### クイックスタート

上記の手順で `derive` を有効にして依存を追加した後、次を `src/main.rs` に保存し、
`cargo run` で実行してください:

```rust
#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = validate_email, error = &'static str)]
struct Email(String);

fn validate_email(value: &str) -> Result<(), &'static str> {
    if value.contains('@') { Ok(()) } else { Err("missing @") }
}

fn main() {
    let email = Email::try_new("a@b.com".to_owned()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(Email::try_new("invalid".to_owned()), Err("missing @"));
}
```

`ValueObject` は `Debug`、`Clone`、`PartialEq`、`Eq`、`Hash` を生成するため、
重複して derive しないでください。検証なしの構造体では `new`、検証付きでは
`try_new` を使います。名前付き・複数フィールドの構造体と、データを持つ列挙型にも対応します。
`Entity` は `#[entity(id)]` フィールドだけを使った等値比較とハッシュを生成します。
任意の `#[entity(validate = check, error = Error, serde)]` で、検証付きの
`try_new(...)` と serde 対応を追加できます（ファサードの `serde` feature が必要）。
検証関数は `&Self` を受け取り、逆直列化時にも実行します。指定された ID を維持し、
作成イベントは発生させません。状態変更メソッド自身でも不変条件を守ってください。
詳細は[Entity の検証と serde](crates/macro/README.ja.md#エンティティと集約イベント)を参照してください。

## 次のステップ: 実行可能なサンプル

**DDD が初めてなら、次は [注文の入門ガイド](examples/README.ja.md#初めての方へ)へ進んでください**。
注文の業務ルールをコードで表し、集約の保存・通知につなぐ流れを学びます。
ガイドでは用語とコードの対応、各サンプルの読む順番も説明します。
チェックアウト後、まずは次を実行してください:

```sh
cargo run -p ddd-toolkit --example getting_started --features derive --locked
```

次に [CQRS](examples/cqrs.rs) でコマンド／クエリハンドラー、Specification と Policy を試せます
（`cargo run -p ddd-toolkit --example cqrs --features derive --locked`）。
どちらも `ddd_toolkit::` API のみを使い、assert で結果を確認します。
再試行・イベントソーシング・アウトボックスの例は [Advanced（発展）](#advanced発展)を参照してください。

## Advanced（発展）

以下の契約は必要に応じて使ってください。四つの derive を使う前提条件ではありません:

- `Repository` / `AsyncRepository`、`DomainEventPublisher` /
  `AsyncDomainEventPublisher`、`dispatch_events` / `dispatch_events_async`。
- CQRS のコマンド／クエリハンドラーとライフサイクルコンテキスト。
- イベントストア、リビジョンの型、集約の再生、スナップショット。
- イベント処理、プロジェクション、チェックポイント、トランザクショナルアウトボックスの契約。
- 仕様、ドメインサービス／ポリシー／ファクトリー、プロセスマネージャー、
  Unit of Work、イベントのアップキャスター。
- 書式出力の秘匿化と破棄時のゼロ化を行う `Secret<T>`。

コアが定義するのは契約です。ストレージアダプター、時計、ID 生成器、非同期ランタイムは
提供せず、トランザクションや配信も自動的には保証しません。アダプターはアプリケーション側で
実装してください。非同期ポートは、ランタイムを選ばずにボックス化された `Send` Future を返します。
通常の `Box::pin` 実装ではサイズがゼロでない Future のためにヒープ確保を行い、
`dyn Future` の poll には動的ディスパッチが使われます。型消去はポートの `dyn` 利用を可能にし、
明示的なライフタイムと `Send` 境界は借用とスレッド安全性の要件を示します。
これは実測された性能問題ではなくトレードオフです。ランタイム非依存でも `!Send` な Future は
扱えず、単一スレッド executor でも現在のシグネチャに適合しません。詳細は
[core の非同期契約](crates/core/README.ja.md#構成要素)を参照してください。

### 発展サンプル

チェックアウト後、`ddd_toolkit::` API のみを使い、assert で結果を確認する次の例を実行できます。

| サンプル | 内容 | コマンド |
| --- | --- | --- |
| [再配信（発展）](examples/dispatch_retry.rs) | 集約、Repository、配信失敗と再試行による重複 | `cargo run -p ddd-toolkit --example dispatch_retry --features derive --locked` |
| [イベントソーシング](examples/event_sourcing.rs) | ExpectedRevision、アップキャスト、再生、スナップショット | `cargo run -p ddd-toolkit --example event_sourcing --features derive --locked` |
| [配信信頼性](examples/reliable_delivery.rs) | 非同期アウトボックス、冪等なプロジェクション、チェックポイント | `cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked` |

すべて `derive`（既定で有効）が必要です。追加依存や非同期ランタイムは不要です。
アダプターはインメモリの実演用で永続ストレージではありません。再試行で重複配信が起こり得ます。
原子的なトランザクション、永続性、claim の復旧は本番アダプターの責任です。
非同期の例は即座に完了する Future のみを poll し、実際の I/O は扱いません。

### 秘密値

```rust
fn main() {
    let key = ddd_toolkit::Secret::new("sekret".to_owned());
    assert_eq!(format!("{key:?}"), "[REDACTED]");
    assert_eq!(key.expose_secret(|value| value.len()), 6);
}
```

### 集約イベントの発行

```rust
use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, dispatch_events};

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct OrderPlaced;

#[derive(ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<OrderPlaced>,
}

struct Publisher;

impl DomainEventPublisher for Publisher {
    type Event = OrderPlaced;
    type Error = &'static str;

    fn publish(&mut self, _event: &OrderPlaced) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let mut order = Order { id: 1, events: vec![] };
    order.record_event(OrderPlaced);
    dispatch_events(&mut order, &mut Publisher).unwrap();
    assert!(order.pull_events().is_empty());
}
```

リポジトリによる永続化とイベント発行は別の操作です。`Repository::save` は集約を
不変で借用し、保留イベントを自動的には取り出しません。`dispatch_events` は成功時に
イベントを取り出してキューを空にし、発行失敗時には取り出した**すべての**イベントを復元します。
発行済みのイベントはロールバックしないため、再試行で重複配信される可能性があります。
トランザクションの保証はアプリケーション側のアダプターが定めます。

## Feature

| Feature | デフォルト | 有効になる機能 |
|---------|---------|---------|
| `derive` | 有効 | `ValueObject`、`Entity`、`DomainEvent`、`AggregateRoot` の derive。コアの直接依存は不要 |
| `serde` | 無効 | コアの serde 再エクスポート、`Secret<T>` の復元、`#[value_object(serde)]` に必要な serde derive 対応。`derive` とは独立 |
| `serde_with` | 無効 | `serde` を含み、合成可能な明示的平文出力用の `ExposeSecret` を追加 |

値オブジェクトをシリアライズするには、ファサードの `serde` feature を有効にし、
型ごとに次の属性を指定します。

```rust
#[derive(ddd_toolkit::ValueObject)]
#[value_object(serde)]
struct UserId(String);
```

生成された実装のために `serde` へ直接依存する必要はありません。
検証付きのデシリアライズでは検証関数を実行し、失敗を `value object validation failed` に
変換します。列挙型は外部タグ形式を使います。同じ属性で検証と併用するときは
`#[value_object(validate = check, error = Error, serde)]` のように `serde` を最後に置いてください。
マクロ 0.4.1 は、フラグの後ろに他のオプションを記述すると拒否します。

単一フィールドのタプルバリアントは Serde の newtype 形式を使います。
`Code(u32)` は `{"Code":7}` を出力し、往復変換できます。コンテナーのフィールドは
自身の形状を維持するため、`Codes(Vec<u32>)` は `{"Codes":[7,8]}` になります。
ユニット、名前付き、複数フィールドのタプルバリアントの形式は変わりません。

### マクロ 0.4 の列挙型保存データからの移行

ファサード 0.4.0 / マクロ 0.5.0 では、patch ではなく破壊的変更として出力形式を
変更します。旧シリアライザーは余分なタプルのラッパーを付けて `{"Code":[7]}` を
出力していましたが、旧デシリアライザーも newtype 形式を期待していました。
保存データを読み込む前に、対象の単一フィールドタプルバリアントだけ外側の配列を
1段取り除いてください。例: `{"Code":[7]}` → `{"Code":7}`、
`{"Codes":[[7,8]]}` → `{"Codes":[7,8]}`。スキーマやバージョン情報で旧レコードを
識別し、すべての配列を一律に変換しないでください。コンテナーを内包する場合は
新旧の表現が曖昧になるため、旧形式への自動フォールバックは行いません。
送受信側の更新を調整するか、移行準備ができるまでファサード 0.3 / マクロ 0.4 に
固定してください。他の Serde フォーマットでも newtype と tuple の違いがあるため、
各フォーマットの API を使って移行してください。

マクロ 0.5.1 は、検証付きコンストラクターや名前付き enum バリアントを含め、
名前付き値オブジェクトの生成コードでフィールド省略記法を使います。
`clippy::redundant_field_names` の許可は不要です。対応バージョンや serde feature の
要件は [core/macro の互換性契約](docs/macro-compatibility.md) を参照してください。

`Secret<T>` は feature の指定なしで使え、`Serialize` は実装しません。
`serde` feature は `Deserialize` を、`serde_with` feature は明示的な平文出力アダプター
`ExposeSecret` を追加します。合成例、エラー秘匿と入出力のゼロ化の限界は
[Secret の serde 連携](docs/secret-serde.md)を参照してください。
`Secret::into_inner` を呼び出した後のゼロ化は呼び出し側の責任です。

## core/macro 0.3 からの移行

- `domain`、`port`、`adapter`、`mock`、`application` のモジュールパスをルートからの
  インポートに置き換えてください。組み込みアダプター／テストダブルと `UseCase` は廃止されました。
- `EntityId` / `EnumVo` の derive を適切な `ValueObject` 型に置き換えてください。
  `Wrapped` の代わりに `ValueObject` の `as_inner` / `into_inner` を使います。
- `#[vo(validate = "function")]` を
  `#[value_object(validate = function, error = YourError)]` に置き換えてください。
  `ValidationError` は廃止され、エラー型は利用側で選びます。
- 生成された `TryFrom` / `From` / `Display` の使用を、コンストラクターや必要に応じた
  アプリケーション側の明示的な実装に置き換えてください。
- `SecretVo` / `#[vo(zeroize)]` を `Secret<T>` に置き換えてください。
- `record` / `take_events` を `record_event` / `pull_events` に置き換えてください。
- `Load` / `Save` / `Delete` を `Repository` または `AsyncRepository` に、
  `EventDispatcher` をイベントパブリッシャーの契約に置き換えてください。
  `PortError` は廃止され、エラー型はアダプターが選びます。
- `chrono`、`uuid`、`zeroize` の feature 指定を削除してください。
- serde は値オブジェクトごとに有効にしてください。derive はファサードだけの依存で利用できます。

詳細は [API ドキュメント](https://docs.rs/ddd-toolkit)を参照してください。
[`facade-tests`](facade-tests/tests) では、独自リポジトリ、非同期ポート、複数集約の処理、
発行失敗と再試行、derive、serde、秘密値をファサード経由で検証しています。

## 変更履歴

[CHANGELOG.md](CHANGELOG.md)を参照してください。

## ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) または [MIT license](LICENSE-MIT) の
いずれかを選択して利用できます。
