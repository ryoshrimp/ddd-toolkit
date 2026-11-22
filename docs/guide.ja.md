# 入門ガイド

[English](guide.md) | 日本語

ddd-toolkit は段階的に導入できます。**多くのアプリケーションではステージ 1 だけで
十分で、以降のステージはすべて任意です。** 各ステージの「次へ進む目安」に当てはまる
問題が出たときだけ進んでください。公開項目を領域別にまとめた API マップは
[クレートのドキュメント](https://docs.rs/ddd-toolkit)にあり（docs.rs には次のリリースから載ります。それまでは `src/lib.rs` を見るか `cargo doc --open` で確認してください）、API ごとの実行例と契約テストは
[公開契約の対応表](public-contracts.md#日本語)で探せます。

コマンドはすべてリポジトリのルートで実行します。

## ステージ 1: 最小構成

**できること:** 値オブジェクト、エンティティ、集約ルートでビジネスルールを表現し、
ドメインイベントを集約内に記録し、リポジトリ経由で保存・読み込みします。

**使う API:** `ValueObject`、`Entity`、`AggregateRoot`、`DomainEvent`、`Repository`
（非同期のストレージには `AsyncRepository`）。

**読む・動かす:** [注文の入門ガイド](../examples/README.ja.md#初めての方へ)と
[getting_started.rs](../examples/getting_started.rs)。

```sh
cargo run -p ddd-toolkit --example getting_started --features derive --locked
```

**次へ進む目安:** 記録したイベントに他のコードが反応する必要が出たとき。

## ステージ 2: イベントの発行

**できること:** 集約が記録したイベントを発行し、配送に成功したら消去します。
一部失敗後の再試行の挙動もこのステージで扱います。

**使う API:** `DomainEventPublisher`、`dispatch_events`。非同期版は
`AsyncDomainEventPublisher`、`dispatch_events_async`。

**読む・動かす:** [dispatch_retry.rs](../examples/dispatch_retry.rs)。

```sh
cargo run -p ddd-toolkit --example dispatch_retry --features derive --locked
```

**次へ進む目安:** 更新と参照を分けたい、または保存と発行の順序を明示したいとき。

## ステージ 3: CQRS

**できること:** コマンドとクエリを分け、保存と発行をライフサイクルコンテキストの中で
実行し、`Specification`（条件の判定）と `Policy`（理由付きで制約を強制）で不正な入力を
拒否します。

**使う API:**（モジュール `cqrs`、feature `cqrs`）
`CommandHandler`、`QueryHandler`、`AggregateCreator`、
`LifecycleContext` と関連する型、`Specification`、`Policy`
（各ハンドラーとコンテキストに非同期版があります。`AggregateCreator` は例では使いません。[public-contracts.md](public-contracts.md#日本語)を参照）。

**読む・動かす:** [cqrs.rs](../examples/cqrs.rs)。

```sh
cargo run -p ddd-toolkit --example cqrs --features derive --locked
```

**次へ進む目安:** 最新状態だけでなく、変更の全履歴が必要になったとき。

## ステージ 4: イベントソーシング

**できること:** イベントを正の情報源として保存し、期待リビジョンで追記を保護し、
リプレイで集約を復元し、スナップショットでリプレイを短縮し、アップキャスターで
古いイベントのバージョンを変換します。

**使う API:**（モジュール `event_sourcing`、feature `event_sourcing`）
`EventStore`（`AsyncEventStore`）、`ExpectedRevision`、`StreamRevision` など
のリビジョン型、`AggregateReplay`、`replay_aggregate`、`SnapshotStore`、
`AggregateSnapshot`、`EventUpcaster`、`UpcasterChain`（`UpcasterChain` は例では使いません。
[public-contracts.md](public-contracts.md#日本語)を参照）。

**読む・動かす:** [event_sourcing.rs](../examples/event_sourcing.rs)。

```sh
cargo run -p ddd-toolkit --example event_sourcing --features derive --locked
```

**次へ進む目安:** イベントを他のシステムやリードモデルへ確実に届ける必要が出たとき。

## その先

- **確実な配送:** トランザクショナルアウトボックス、イベントコンシューマー、
  プロジェクション、チェックポイント、冪等なプロジェクション
  （モジュール `delivery`、feature `delivery`）。
  [reliable_delivery.rs](../examples/reliable_delivery.rs)を参照してください
  （`cargo run -p ddd-toolkit --example reliable_delivery --features derive --locked`）。
- **その他の構成要素:** `DomainService`、`DomainFactory`、`UnitOfWork`、
  `process::ProcessManager`（feature `process`）。
  実行例と契約テストは [public-contracts.md](public-contracts.md#日本語)を
  参照してください。
- **シークレット:** `secret::Secret`（feature `secret`、既定で有効）は書式出力を秘匿化し、破棄時にゼロ化します。
  直列化は [secret-serde.md](secret-serde.md)を参照してください。
