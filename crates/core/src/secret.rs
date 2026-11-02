use core::fmt;
use core::mem::ManuallyDrop;
use zeroize::Zeroize;

/// **English:**<br>
/// Owns a value whose [`Zeroize::zeroize`] method is called before it is dropped.
///
/// `Debug` and `Display` write the literal `[REDACTED]` rather than formatting `T`. Access is explicit through
/// [`Self::as_inner`], [`Self::expose_secret`], or [`Self::into_inner`]; this type implements
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
/// Cloning copies the inner value into another independently owned `Secret`. Equality delegates
/// to the inner type and does not promise constant-time comparison.
///
/// Zeroization follows `T`'s implementation and requires this wrapper's destructor to run;
/// it does not erase copies made elsewhere. [`Self::into_inner`] bypasses that destructor.
/// If `T::zeroize` panics, the inner value's destructor is not called by this wrapper.
/// `Send` and `Sync` depend on `T`; no synchronization is added.
///
/// Formatting hides the value; the closure below explicitly reads its length.
///
/// **日本語:**<br>
/// 所有する値を破棄する前に、その値の [`Zeroize::zeroize`] を呼び出すラッパーです。
///
/// `Debug` と `Display` は `T` を書式出力せず、リテラル `[REDACTED]` を出力します。値へのアクセスには
/// [`Self::as_inner`]、[`Self::expose_secret`]、[`Self::into_inner`] を明示的に使います。
/// `Deref` と serde の `Serialize` は実装しません。`serde` feature では
/// `T: serde::Deserialize<'de> + Zeroize` に対する `Deserialize` を実装し、[`Self::new`] で
/// 所有権を受け取ります。`serde_with` feature の `ExposeSecret` で、フィールドの
/// 平文出力を明示的に許可できます。内部型由来の復元エラーは `failed to deserialize secret` に
/// 置き換えますが、フォーマット側が独立に生成するエラーや呼び出し側・独自実装のログは秘匿しません。
/// 暗号化ではありません。正常に包んだ値は破棄時にゼロ化しますが、入出力バッファ、パーサーの
/// 一時値、構築途中・失敗時の値や別のコピーは対象外です。失敗時の後始末は内部型とパーサーに
/// 依存し、serde の契約ではエラーから構築途中の `T` を回収できません。入力バッファを
/// `zeroize::Zeroizing` で管理すると軽減できますが、パーサーのコピーは消去できません。
/// クローンは内包値を複製して別の `Secret` に所有させます。等値比較は内包型に
/// 委譲し、定数時間での比較を保証しません。
///
/// ゼロ化の範囲は `T` の実装に従い、このラッパーのデストラクターが実行される必要があります。
/// 別の場所に作られたコピーは消去しません。[`Self::into_inner`] はデストラクターを迂回します。
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
    pub fn as_inner(&self) -> &T {
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
    /// let mut value = Secret::new(vec![1_u8, 2, 3]).into_inner();
    /// assert_eq!(value, [1, 2, 3]);
    /// value.zeroize();
    /// ```
    pub fn into_inner(self) -> T {
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
        f(self.as_inner())
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

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.value.zeroize();
        // SAFETY: The value is initialized and has not been taken. Drop runs once,
        // and ManuallyDrop prevents an automatic second drop of the inner value.
        unsafe { ManuallyDrop::drop(&mut self.value) };
    }
}

impl<T: Zeroize + Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self::new(self.as_inner().clone())
    }
}

impl<T: Zeroize + PartialEq> PartialEq for Secret<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_inner() == other.as_inner()
    }
}

impl<T: Zeroize + Eq> Eq for Secret<T> {}

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
