# Secret serde integration / Secret の serde 連携

## English

`Secret<T>` is available without features. The `serde` feature adds
`Deserialize` for `T: Deserialize<'de> + zeroize::Zeroize`; values are wrapped
with `Secret::new`. Deriving `Deserialize` for structs containing secrets,
including ordinary serde `Option` and `Vec` compositions, needs no `serde_with`.
`Secret<T>` never implements `Serialize`: deriving it on a containing struct
cannot accidentally expose a secret.

For explicit output, enable the facade's `serde_with` feature (which includes
`serde`) and depend on `serde` with `derive` and `serde_with` with `macros`.
The core crate exposes the same feature and root-level `ExposeSecret` adapter;
replace `ddd_toolkit` with `ddd_toolkit_core` in the example below.
`ExposeSecret` implements `SerializeAs<Secret<T>>` for
`T: Serialize + zeroize::Zeroize`, delegating to the inner value. It is an
output-only adapter; deserialization uses `Secret` directly.

```toml
[dependencies]
ddd-toolkit = { version = "0.4", features = ["serde_with"] }
serde = { version = "1", features = ["derive"] }
serde_with = { version = "3.24.0", features = ["macros"] }
```

Use the shared Rust example below for a field-level opt-in. The adapter composes
with containers supported by `serde_with`, including `Option`, `Vec`,
`BTreeMap<String, Secret<String>>` via `BTreeMap<_, ExposeSecret>`, and nested
`Option<Vec<Secret<String>>>`. This does not promise support for every serde
container. Custom containers may need a user-defined adapter or conversion to a
storage DTO; there are no container-specific output functions.

Inner deserialization errors are replaced with `failed to deserialize secret`
without formatting the original error. Format-level errors generated independently,
and logs from callers or custom deserializers, are not covered. Serialization
errors are propagated unchanged and may contain secrets.

This is **plaintext output, not encryption**. Secret only handles redacted
formatting and zeroization of successfully wrapped values on drop. Output bytes,
input buffers, parser temporaries, partially constructed/failed values and other
copies are not zeroized by dropping the original secret. Serde's `Deserialize`
contract cannot recover a partial `T` from `Err`; cleanup depends on the inner
type and parser. Managing an input buffer with `zeroize::Zeroizing` can reduce
exposure, but cannot guarantee erasure of parser/internal copies.

For recoverable credentials, encrypt in an application persistence adapter
(such as a concrete Repository) and convert to a storage DTO as needed. Key
management and encryption are separate responsibilities. For password checking,
use a dedicated password hash rather than persisting plaintext passwords.

## 日本語

`Secret<T>` 自体に feature は不要です。`serde` feature では
`T: Deserialize<'de> + zeroize::Zeroize` に対して `Deserialize` を実装し、
`Secret::new` で値を包みます。秘密値を含む構造体の `Deserialize` derive や、通常の
serde による `Option`・`Vec` の合成には `serde_with` は不要です。
`Secret<T>` 自体は `Serialize` を実装しないため、包含構造体の derive によって
意図せず秘密値を出力することはできません。

明示的な出力には facade の `serde_with` feature（`serde` も有効化）を使い、
`derive` 付きの `serde` と `macros` 付きの `serde_with` に依存してください。
上記の Cargo 設定が使えます。core クレートにも同じ feature とルートの
`ExposeSecret` があり、下記の例の `ddd_toolkit` を `ddd_toolkit_core` に変更できます。
`ExposeSecret` は `T: Serialize + zeroize::Zeroize` に対して
`SerializeAs<Secret<T>>` を実装し、内部値に出力を委譲します。
入力用アダプターではなく、復元には `Secret` 自体の実装を使います。

下記の共有例でフィールドごとの出力を許可します。`serde_with` が対応する
`Option`、`Vec`、`BTreeMap<_, ExposeSecret>` を使った
`BTreeMap<String, Secret<String>>`、入れ子の `Option<Vec<Secret<String>>>`
などと合成できます。serde が扱う全コンテナへの対応を保証するものではありません。
独自コンテナには利用者側のアダプターや保存用 DTO への変換が必要な場合があります。
コンテナ別の専用出力関数は提供しません。

内部型のデシリアライズエラーは、元のエラーを書式出力せず、固定メッセージ
`failed to deserialize secret` に置き換えます。フォーマット側が独立に生成するエラーや、
呼び出し側・独自デシリアライザーのログは保証範囲外です。
シリアライズエラーはそのまま伝播し、秘密値を含む可能性があります。

これは**平文出力であり、暗号化ではありません**。Secret の責務は表示の秘匿化と、
正常に包んだ値の破棄時のゼロ化です。出力、入力バッファ、パーサーの一時値、
構築途中・失敗時の値、別のコピーは元の秘密値の破棄ではゼロ化しません。
serde の `Deserialize` の契約では `Err` から構築途中の `T` を回収できず、
後始末は内部型とパーサーに依存します。入力を `zeroize::Zeroizing` で管理すれば
軽減できますが、パーサーや内部型のコピーまで完全に消去する保証はありません。

復元が必要な認証情報は、必要に応じて Repository 具象実装などの永続化アダプターで
暗号化し、保存用 DTO に変換します。鍵管理・暗号化は別の責務です。
パスワード照合には平文の永続化ではなく専用のパスワードハッシュを使ってください。

## Example / 使用例

```rust
use ddd_toolkit::{ExposeSecret, Secret};
use serde_with::serde_as;

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredCredentials {
    #[serde_as(serialize_as = "ExposeSecret")]
    token: Secret<String>,
    #[serde_as(serialize_as = "Option<Vec<ExposeSecret>>")]
    tokens: Option<Vec<Secret<String>>>,
}
```
