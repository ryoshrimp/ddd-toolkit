use crate::Secret;
use zeroize::Zeroize;

/// **English:**<br>
/// Explicitly serializes a `Secret<T>` as its unredacted inner value with `serde_with`.
///
/// Available with the `serde_with` feature (which enables `serde`). Implements only
/// `serde_with::SerializeAs`; use `Secret`'s ordinary `Deserialize` for input. Compose this
/// adapter with containers supported by `serde_with`, such as `Option< Vec<ExposeSecret> >`.
/// Custom containers may need a user-defined adapter or a storage DTO.
/// Output is plaintext, not encrypted, and is not zeroized when the source secret is dropped.
/// Inner serialization errors are propagated unchanged and may also contain secrets.
///
/// **日本語:**<br>
/// `serde_with` を使い、`Secret<T>` の内部値を秘匿せず明示的にシリアライズします。
///
/// `serde_with` feature（`serde` も有効化）が必要です。`serde_with::SerializeAs` のみを実装し、
/// 入力には `Secret` 自体の通常の `Deserialize` を使います。`Option< Vec<ExposeSecret> >` など、
/// `serde_with` が対応するコンテナと合成できます。独自コンテナには利用者定義のアダプターや
/// 保存用 DTO が必要な場合があります。出力は暗号化されない平文であり、元の秘密値を破棄しても
/// ゼロ化されません。内部型のシリアライズエラーはそのまま伝播し、秘密値を含む可能性もあります。
///
/// # Example / 使用例
///
/// ```rust
/// # #[cfg(feature = "serde_with")] {
/// use ddd_toolkit_core::{ExposeSecret, Secret};
/// use serde_with::SerializeAs;
///
/// let token = Secret::new(String::from("storage-token"));
/// let mut bytes = Vec::new();
/// let mut serializer = serde_json::Serializer::new(&mut bytes);
/// ExposeSecret::serialize_as(&token, &mut serializer).unwrap();
/// assert_eq!(bytes, br#""storage-token""#);
/// # }
/// ```
pub struct ExposeSecret;

impl<T: Zeroize + serde::Serialize> serde_with::SerializeAs<Secret<T>> for ExposeSecret {
    fn serialize_as<S: serde::Serializer>(
        source: &Secret<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        source.as_inner().serialize(serializer)
    }
}
