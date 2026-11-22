# Versioning policy

This page describes how `ddd-toolkit` (facade), `ddd-toolkit-core` and
`ddd-toolkit-macro` are versioned while they are 0.x. For the exact
core/macro support range of generated code, see
[Generated-code compatibility](macro-compatibility.md); for the release
procedure, see [Releasing the workspace](releasing.md).

This policy applies starting after the next release. The breaking changes listed
under `[Unreleased]` in the changelogs (#60, #64, #66, #68, #69, #70, #73), some of
which remove APIs without deprecated aliases, are exempt.

## Breaking changes in 0.x

- An incompatible change to a crate's public API bumps that crate's **minor**
  version (0.x.y -> 0.(x+1).0). Patch releases are compatible only.
- What counts as a breaking change follows the
  [Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html).
- Versions are independent per crate; the numbers need not match.
- When core or macro gets a minor bump that the facade re-exports or depends
  on, the facade's minor version is bumped too. See
  [Releasing](releasing.md#generated-code-compatibility-gate) and
  [Generated-code compatibility](macro-compatibility.md#api-stability-for-generated-code).

## Deprecation

A public API is first marked `#[deprecated]` (naming its replacement) in at
least one released minor line before it is removed. Removal happens in a later
minor release at the earliest: for example, deprecated in 0.4.x, removable from
0.5.0. There is no time-based minimum.

## Stability tiers and features

The crate root of `ddd-toolkit-core` (and the facade) keeps the central items:
`ValueObject`, `Entity`, `AggregateRoot`, `DomainEvent` and its helpers, the
repository and publisher ports with `dispatch_events`, `UnitOfWork`,
`Specification`, `DomainService`, `Policy`, `DomainFactory`. Items in the
`cqrs`, `event_sourcing`, `delivery`, `process`, and `secret` modules are
**experimental**: they may change during 0.x. Each module is behind a feature of the same name,
enabled by default (`delivery` includes `event_sourcing`; `secret` is also
required by `serde_with`). Core's `subtle` and `zeroize` dependencies are only
pulled in by `secret`.

## MSRV policy

From the next release, the MSRV is **1.88** (the workspace `rust-version`,
verified by a CI job; currently under `[Unreleased]`, #72). Releases v0.3.1 to
v0.4.7 declare no `rust-version`, and v0.1.1 to v0.3.0 declared 1.85. Raising
the MSRV is treated like a breaking change: it bumps the minor version and
never happens in a patch release. No fixed "N releases behind stable" window is
promised.

## Supported lines

During 0.x only the latest minor line of each crate receives fixes. There are
no backports.

## Version correspondence

Requirements below are taken from each facade line's manifest at the git tags
`v0.1.0` ... `v0.4.7`. The core and macro requirement lines did not change
within any facade minor line (only the patch lower bounds moved).

| Facade line | Core requirement | Macro requirement |
| --- | --- | --- |
| 0.4.x (current: 0.4.7) | ^0.4 (0.4.2 at 0.4.0; 0.4.6 at 0.4.7) | ^0.5 (0.5.0 at 0.4.0; 0.5.5 at 0.4.7) |
| 0.3.x | ^0.4 (0.4.0 at 0.3.0; 0.4.1 at 0.3.1) | ^0.4 (0.4.0 at 0.3.0; 0.4.1 at 0.3.1) |
| 0.2.x | ^0.3 (0.3.0) | ^0.3 (0.3.0) |
| 0.1.x | ^0.2 (0.2.0 at 0.1.0; 0.2.2 at 0.1.5) | ^0.2 (0.2.1 at 0.1.0; 0.2.3 at 0.1.5) |

The tags before 0.3.1 predate the `crates/` layout and the workspace
dependency table, so those requirements come from the facade's own
`[dependencies]`.

Current recommended combination: facade 0.4.7 with core 0.4.6 and macro 0.5.5.
For the precise support range when depending on core and macro directly, see
[Generated-code compatibility](macro-compatibility.md#supported-combinations).

## 日本語

このページは、0.x の間の `ddd-toolkit`（facade）、`ddd-toolkit-core`、
`ddd-toolkit-macro` のバージョニング方針を説明します。生成コードにおける core/macro
の正確なサポート範囲は[生成コードの互換性](macro-compatibility.md#日本語)、リリース手順は
[リリース手順](releasing.md)を参照してください。

この方針は次のリリースの後から適用します。changelog の `[Unreleased]` に記載された
破壊的変更（#60, #64, #66, #68, #69, #70, #73。非推奨エイリアスなしで API を削除するものを含む）は対象外です。

### 0.x での破壊的変更

- クレートの公開 API への非互換な変更は、そのクレートの **minor** バージョンを上げます
  （0.x.y -> 0.(x+1).0）。patch リリースは互換性のある変更のみです。
- 何が破壊的変更にあたるかは
  [Cargo の SemVer リファレンス](https://doc.rust-lang.org/cargo/reference/semver.html)に従います。
- バージョンはクレートごとに独立しており、番号を揃える必要はありません。
- facade が再エクスポートまたは依存している core / macro の minor が上がる場合は、
  facade の minor も上げます。詳細は
  [リリース手順](releasing.md#generated-code-compatibility-gate)と
  [生成コードの互換性](macro-compatibility.md#api-stability-for-generated-code)を参照してください。

### 非推奨化

公開 API は、削除する前に、少なくとも 1 つのリリース済み minor ラインで、代替を明記した
`#[deprecated]` を付けます。削除は早くても後続の minor リリースで行います（例: 0.4.x で
非推奨化した場合、0.5.0 以降で削除可能）。期間による最小保証はありません。

### 安定性の区分と feature

`ddd-toolkit-core`（およびファサード）のルートには中核の項目だけを残します:
`ValueObject`、`Entity`、`AggregateRoot`、`DomainEvent` とその補助、リポジトリと
パブリッシャーのポートおよび `dispatch_events`、`UnitOfWork`、`Specification`、
`DomainService`、`Policy`、`DomainFactory`。`cqrs`、`event_sourcing`、`delivery`、
`process`、`secret` の各モジュールの項目は**実験的**で、0.x の間は変わる可能性があります。各モジュールは同名の feature（既定で有効）の背後にあり、
`delivery` は `event_sourcing` を含み、`secret` は `serde_with` にも必要です。
core の `subtle` と `zeroize` は `secret` のときだけ依存に加わります。

### MSRV 方針

次のリリースから、MSRV は **1.88**（workspace の `rust-version`。CI ジョブで検証。現在は
`[Unreleased]`、#72）です。v0.3.1〜v0.4.7 は `rust-version` を宣言しておらず、
v0.1.1〜v0.3.0 は 1.85 を宣言していました。
MSRV の引き上げは破壊的変更と同様に扱い、minor を上げます。patch では引き上げません。
「stable から N リリース前まで」のような固定の期間は約束しません。

### サポート対象ライン

0.x の間は、各クレートの最新の minor ラインのみ修正を受けます。バックポートは行いません。

### バージョン対応表

下表は git タグ `v0.1.0` ... `v0.4.7` 時点の facade のマニフェストにある要件です。
いずれの facade minor ライン内でも、core/macro の要件ライン自体は変わっていません
（patch の下限のみ変化）。

| facade ライン | core の要件 | macro の要件 |
| --- | --- | --- |
| 0.4.x（現行: 0.4.7） | ^0.4（0.4.0 時点 0.4.2、0.4.7 時点 0.4.6） | ^0.5（0.4.0 時点 0.5.0、0.4.7 時点 0.5.5） |
| 0.3.x | ^0.4（0.3.0 時点 0.4.0、0.3.1 時点 0.4.1） | ^0.4（0.3.0 時点 0.4.0、0.3.1 時点 0.4.1） |
| 0.2.x | ^0.3（0.3.0） | ^0.3（0.3.0） |
| 0.1.x | ^0.2（0.1.0 時点 0.2.0、0.1.5 時点 0.2.2） | ^0.2（0.1.0 時点 0.2.1、0.1.5 時点 0.2.3） |

0.3.1 より前のタグは `crates/` 構成や workspace 依存テーブルより前のものなので、
facade 自身の `[dependencies]` から読み取っています。

現在の推奨の組み合わせは facade 0.4.7 / core 0.4.6 / macro 0.5.5 です。
core と macro を直接依存にする場合の正確なサポート範囲は
[生成コードの互換性](macro-compatibility.md#日本語)を参照してください。
