# Generated-code compatibility

## Supported combinations

- Macro **0.5.x** supports core **>=0.4.1, <0.5.0** for direct external consumers.
  The tested lower bound is the exact crates.io release **0.4.1**; the other
  endpoint is the current workspace core (0.4.4). Intermediate patch releases
  must preserve the same contracts. This is not a Rust MSRV guarantee: tests
  run on latest stable with matching rust-src.
- The current facade **0.4.2** selects core **^0.4.4** and macro **^0.5.1**.
  Prefer this facade combination unless you need separate dependencies.
  The facade re-exports/derives are additionally tested against core 0.4.1
  using its actual sources and a staged manifest with only core selection
  changed. That is a lower-bound compatibility test, not its published resolver
  requirement. No global Cargo patch substitutes the current core for 0.4.1.
- Both direct and facade access, original and renamed dependencies, and serde
  enabled/disabled are tested in independent Cargo workspaces by
  `scripts/test_macro_consumers.py`. Serde cases exercise runtime JSON
  round-trips (including newtype enum variants); plain cases exercise all four
  derives. Minimum-core cases may access crates.io to fetch that exact release.
  Current-core and missing-feature cases use local paths, isolated from the
  macro crate's serde-enabled dev-dependency.
- If both core and facade are dependencies, derives prefer the direct core.
  Enable serde on that selected core; facade-only users enable facade's `serde`.
  Macro has no `serde` feature. See the macro's [serialization guide](../crates/macro/README.md#serialization).

Core 0.4.0 and earlier, core 0.5 and later, and unrelated macro minor lines are
outside this contract, even if some programs happen to compile. Upgrade to the
current facade, or select core 0.4.4 and macro 0.5.1 explicitly. Macro 0.4.x has
the older single-field enum serialization format; migrating to 0.5.x requires
reviewing persisted data as described in the macro changelog. A later minor
line must declare its own tested core interval.

## API stability for generated code

`__private` is hidden from ordinary API documentation, but its **serde-enabled
`__private::serde` re-export** and the signatures/semantics of `ValueObject`,
`Entity`, `DomainEvent`, and `AggregateRoot` are part of the generated-code
compatibility contract. Do not remove, rename, narrow, or incompatibly change
these in a core 0.4 patch. Additive helpers do not allow a macro to silently
raise its minimum supported core: preserve an old-code fallback or explicitly
revise the compatibility range in a breaking macro release. An explicit
feature-boundary test also protects ordinary derives without serde.

For incompatible traits, helper paths, or generated serialization behavior in
0.x, bump the affected crate's **minor** version (not patch). Update every
macro/facade whose emitted code or re-exports depend on that change, their
requirements and lockfile, and this matrix. Crate version numbers need not
match; their supported combinations must. See [the release checklist](releasing.md).

## 日本語

macro 0.5.x の直接利用は core >=0.4.1, <0.5.0 をサポートします。最低版は
crates.io の正確な 0.4.1、現行版は workspace の 0.4.4 で検証します。
facade 0.4.2 の公開依存は core ^0.4.4 / macro ^0.5.1 です。最低版テストでは
実際の facade ソースと依存選択のみ変更した manifest を使い、公開依存条件と区別します。
直接利用・facade・依存名変更・serde 有無を独立 workspace で検証し、feature 統合での
見かけ上の成功を防ぎます。両方への依存がある場合は直接の core が優先されます。

`__private::serde`（serde 有効時）と生成コードが使うトレイトの契約は、内部名でも
patch で非互換に変更しません。0.x の非互換変更は該当クレートの minor を更新し、
必要な macro/facade の依存条件・バージョン・検証範囲も更新します。番号の統一は不要です。
範囲外の組み合わせは未サポートです。現行 facade、または core 0.4.4 / macro 0.5.1 に
移行してください。macro 0.4.x から 0.5.x では単一フィールド enum の保存形式に注意し、
CHANGELOG の移行手順に従ってください。
