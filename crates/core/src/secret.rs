use core::fmt;
use core::mem::ManuallyDrop;
use zeroize::Zeroize;

/// **English:**<br>
/// Owns a value whose [`Zeroize::zeroize`] method is called before it is dropped.
///
/// `Debug` and `Display` write the literal `[REDACTED]` rather than formatting `T`. Access is explicit through
/// [`Self::expose_secret_ref`], [`Self::expose_secret`], or [`Self::into_exposed_secret`]; this type implements
/// neither `Deref` nor serde `Serialize`. With `serde`, it implements `Deserialize` for
/// `T: serde::Deserialize<'de> + Zeroize`, taking ownership through [`Self::new`]. With
/// `serde_with`, `ExposeSecret` explicitly opts a field into plaintext output.
/// Internal deserialization errors are replaced with `failed to deserialize secret`;
/// independent format errors and logs from callers or custom implementations are not redacted.
/// This is not encryption. Successfully wrapped values are zeroized on drop, but input/output
/// buffers, parser temporaries, partially constructed or failed values, and other copies are not
/// covered. Cleanup during failed deserialization depends on the inner type and parser; serde's
/// contract cannot recover a partial `T` from an error. A caller-owned `zeroize::Zeroizing`
/// input buffer can reduce exposure, but cannot erase parser copies.
/// This type does not implement `Clone`; copy deliberately with [`Self::clone_secret`], which
/// creates another independently owned `Secret`. It implements [`zeroize::ZeroizeOnDrop`].
/// It implements neither `PartialEq` nor `Eq`; compare with [`Self::ct_eq`], which is
/// constant-time for equal lengths but leaks the length.
///
/// Zeroization follows `T`'s implementation and requires this wrapper's destructor to run;
/// it does not erase copies made elsewhere. Only the value `T` currently owned is zeroized.
/// Growing a `String` or `Vec` before passing it to [`Self::new`] frees the old buffers without
/// zeroizing them; where possible, reserve the final capacity up front (such as
/// `String::with_capacity`) before building the value. A move (passing by value, returning)
/// copies `T`'s inline part and does not erase the old location; for types that store the value
/// inline, such as `[u8; N]`, that leaves the secret itself behind. Temporary copies on the
/// stack or in registers are not erased either.
/// [`Self::into_exposed_secret`] bypasses this wrapper's destructor.
/// If `T::zeroize` panics, the inner value's destructor is not called by this wrapper.
/// `Send` and `Sync` depend on `T`; no synchronization is added.
///
/// Formatting hides the value; the closure below explicitly reads its length.
///
/// **日本語:**<br>
/// 所有する値を破棄する前に、その値の [`Zeroize::zeroize`] を呼び出すラッパーです。
///
/// `Debug` と `Display` は `T` を書式出力せず、リテラル `[REDACTED]` を出力します。値へのアクセスには
/// [`Self::expose_secret_ref`]、[`Self::expose_secret`]、[`Self::into_exposed_secret`] を明示的に使います。
/// `Deref` と serde の `Serialize` は実装しません。`serde` feature では
/// `T: serde::Deserialize<'de> + Zeroize` に対する `Deserialize` を実装し、[`Self::new`] で
/// 所有権を受け取ります。`serde_with` feature の `ExposeSecret` で、フィールドの
/// 平文出力を明示的に許可できます。内部型由来の復元エラーは `failed to deserialize secret` に
/// 置き換えますが、フォーマット側が独立に生成するエラーや呼び出し側・独自実装のログは秘匿しません。
/// 暗号化ではありません。正常に包んだ値は破棄時にゼロ化しますが、入出力バッファ、パーサーの
/// 一時値、構築途中・失敗時の値や別のコピーは対象外です。失敗時の後始末は内部型とパーサーに
/// 依存し、serde の契約ではエラーから構築途中の `T` を回収できません。入力バッファを
/// `zeroize::Zeroizing` で管理すると軽減できますが、パーサーのコピーは消去できません。
/// `Clone` は実装せず、複製は [`Self::clone_secret`] で明示的に行い、別の `Secret` に所有させます。
/// [`zeroize::ZeroizeOnDrop`] を実装します。`PartialEq` と `Eq` は実装せず、比較には
/// [`Self::ct_eq`] を使います（同じ長さなら定数時間で、長さは漏れます）。
///
/// ゼロ化の範囲は `T` の実装に従い、このラッパーのデストラクターが実行される必要があります。
/// 別の場所に作られたコピーは消去しません。ゼロ化されるのは現在所有している値 `T` だけです。
/// `String` や `Vec` を [`Self::new`] に渡す前に伸ばすと、古いバッファはゼロ化されずに
/// 解放されます。可能なら `String::with_capacity` などで最終的な容量を確保してから値を作って
/// ください。ムーブ（値渡しや戻り値）は `T` のインライン部分をコピーし、元の場所は消去されません。
/// `[u8; N]` のように値をインラインで保持する型では、秘密値そのものが残ります。スタックや
/// レジスタ上の一時的なコピーも消去されません。
/// [`Self::into_exposed_secret`] はこのラッパーのデストラクターを迂回します。
/// `T::zeroize` がパニックした場合、このラッパーは内包値のデストラクターを呼び出しません。
/// `Send` と `Sync` は `T` に依存し、同期機構は追加しません。
///
/// 書式出力は値を隠します。次のクロージャーでは長さを明示的に読み取ります。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_core::Secret;
///
/// let secret = Secret::new(String::from("token"));
/// assert_eq!(format!("{secret:?}"), "[REDACTED]");
/// assert_eq!(format!("{secret}"), "[REDACTED]");
/// assert_eq!(secret.expose_secret(|value| value.len()), 5);
/// ```
pub struct Secret<T: Zeroize> {
    value: ManuallyDrop<T>,
}

impl<T: Zeroize> Secret<T> {
    /// **English:**<br>
    /// Takes ownership of `value` without copying or zeroizing it at construction.
    ///
    /// **日本語:**<br>
    /// 構築時にコピーやゼロ化を行わず、`value` の所有権を受け取ります。
    pub fn new(value: T) -> Self {
        Self {
            value: ManuallyDrop::new(value),
        }
    }

    /// **English:**<br>
    /// Borrows the unredacted value for no longer than the borrow of `self`.
    ///
    /// Callers can read or copy the value; those copies are not zeroized by this wrapper.
    ///
    /// **日本語:**<br>
    /// `self` の借用期間を超えない参照として、秘匿化していない値を借用します。
    ///
    /// 呼び出し側は値を読み取ったりコピーしたりできますが、そのコピーはこのラッパーではゼロ化しません。
    pub fn expose_secret_ref(&self) -> &T {
        &self.value
    }

    /// **English:**<br>
    /// Consumes the wrapper and returns its value without zeroizing it.
    ///
    /// The wrapper's destructor is not run. The caller becomes responsible for the returned
    /// value's lifetime and any required zeroization.
    ///
    /// **日本語:**<br>
    /// ラッパーを消費し、ゼロ化せずに内包値を返します。
    ///
    /// ラッパーのデストラクターは実行しません。返された値の生存期間と、必要なゼロ化の管理は
    /// 呼び出し側の責任になります。
    ///
    /// # Example / 使用例
    ///
    /// ```rust
    /// use ddd_toolkit_core::Secret;
    /// use zeroize::Zeroize;
    ///
    /// let mut value = Secret::new(vec![1_u8, 2, 3]).into_exposed_secret();
    /// assert_eq!(value, [1, 2, 3]);
    /// value.zeroize();
    /// ```
    #[allow(unsafe_code)]
    pub fn into_exposed_secret(self) -> T {
        let mut this = ManuallyDrop::new(self);
        // SAFETY: The value is still initialized and taken exactly once. The outer
        // ManuallyDrop prevents Secret::drop from accessing or dropping it afterward.
        unsafe { ManuallyDrop::take(&mut this.value) }
    }

    /// **English:**<br>
    /// Calls `f` once with a shared reference to the value and returns its result.
    ///
    /// This does not consume or zeroize the value. The closure can copy it or return a borrowed
    /// reference; the closure is not an isolation boundary. Any panic from `f` propagates.
    ///
    /// **日本語:**<br>
    /// 値への共有参照を渡して `f` を1回呼び出し、その結果を返します。
    ///
    /// 値の消費やゼロ化は行いません。クロージャーは値をコピーしたり借用した参照を返したりでき、
    /// 情報の隔離境界にはなりません。`f` がパニックした場合はそのまま伝播します。
    pub fn expose_secret<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(self.expose_secret_ref())
    }

    /// **English:**<br>
    /// Deliberately creates another independently owned `Secret` holding a clone of the value.
    ///
    /// This increases the number of secret copies in memory. Each copy is zeroized when its own
    /// `Secret` is dropped; copies made elsewhere are not covered.
    ///
    /// **日本語:**<br>
    /// 値のクローンを持つ、独立して所有される別の `Secret` を意図的に作成します。
    ///
    /// メモリ上の秘密値のコピーが増えます。各コピーはそれぞれの `Secret` の破棄時にゼロ化されますが、
    /// 別の場所に作られたコピーは対象外です。
    ///
    /// # Example / 使用例
    ///
    /// ```rust
    /// use ddd_toolkit_core::Secret;
    ///
    /// let original = Secret::new(String::from("token"));
    /// let copy = original.clone_secret();
    /// assert!(original.ct_eq(&copy));
    /// ```
    pub fn clone_secret(&self) -> Self
    where
        T: Clone,
    {
        Self::new(self.expose_secret_ref().clone())
    }

    /// **English:**<br>
    /// Compares two secrets by their `AsRef<[u8]>` byte representation.
    ///
    /// Byte strings of equal length are compared in constant time. If the lengths differ, this
    /// returns `false` immediately, so the length is not hidden.
    ///
    /// **日本語:**<br>
    /// 2つの秘密値を `AsRef<[u8]>` のバイト表現で比較します。
    ///
    /// 同じ長さのバイト列は定数時間で比較します。長さが異なる場合はすぐに `false` を返すため、
    /// 長さは隠せません。
    ///
    /// # Example / 使用例
    ///
    /// ```rust
    /// use ddd_toolkit_core::Secret;
    ///
    /// let token = Secret::new(String::from("token"));
    /// assert!(token.ct_eq(&Secret::new(String::from("token"))));
    /// assert!(!token.ct_eq(&Secret::new(String::from("tokex"))));
    /// ```
    pub fn ct_eq(&self, other: &Self) -> bool
    where
        T: AsRef<[u8]>,
    {
        use subtle::ConstantTimeEq;
        bool::from(
            self.expose_secret_ref()
                .as_ref()
                .ct_eq(other.expose_secret_ref().as_ref()),
        )
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Zeroize + serde::Deserialize<'de>> serde::Deserialize<'de> for Secret<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer)
            .map(Self::new)
            .map_err(|_| serde::de::Error::custom("failed to deserialize secret"))
    }
}

#[allow(unsafe_code)]
impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.value.zeroize();
        // SAFETY: The value is initialized and has not been taken. Drop runs once,
        // and ManuallyDrop prevents an automatic second drop of the inner value.
        unsafe { ManuallyDrop::drop(&mut self.value) };
    }
}

impl<T: Zeroize> zeroize::ZeroizeOnDrop for Secret<T> {}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}
