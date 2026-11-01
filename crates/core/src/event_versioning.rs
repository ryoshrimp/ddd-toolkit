use std::marker::PhantomData;

/// **English:**<br>
/// Event name and schema version supplied by the application.
/// Fields are public and unvalidated: an empty name or version zero is accepted. Equality compares
/// both fields. This value supplies no registry, serialization format, or migration dispatch.
///
/// **日本語:**<br>
/// アプリケーションが指定するイベント名とスキーマバージョンです。
/// フィールドは公開され検証されません。空の名前やバージョンゼロも許容します。等値比較は
/// 両フィールドを比較します。レジストリー、直列化形式、移行の振り分けは提供しません。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventIdentity {
    /// **English:**<br>
    /// Static event name; uniqueness and naming conventions are application-defined.
    ///
    /// **日本語:**<br>
    /// 静的なイベント名です。一意性と命名規則はアプリケーションが定めます。
    pub name: &'static str,

    /// **English:**<br>
    /// Numeric schema version, with no enforced starting value or ordering policy.
    ///
    /// **日本語:**<br>
    /// 数値のスキーマバージョンです。開始値や順序の方針は強制しません。
    pub version: u32,
}

/// **English:**<br>
/// Type-level event identity independent of any particular event instance.
/// No [`crate::DomainEvent`] bound is required, and the trait does not validate the returned identity.
///
/// **日本語:**<br>
/// 個々のイベントインスタンスとは独立した、型単位のイベント識別情報です。
/// [`crate::DomainEvent`] の境界は要求せず、返された識別情報も検証しません。
pub trait VersionedEvent {
    /// **English:**<br>
    /// Returns the name and schema version chosen for this event type.
    /// Implementations define the mapping; no instance data is passed to this function.
    ///
    /// **日本語:**<br>
    /// このイベント型に選んだ名前とスキーマバージョンを返します。
    /// 対応付けは実装が定め、この関数にはインスタンスのデータを渡しません。
    fn identity() -> EventIdentity;
}

/// **English:**<br>
/// Fallible owned transformation from an event representation back to the same Rust type.
/// Versions can be represented inside that type, but checking or advancing them is the
/// implementation's responsibility. No [`VersionedEvent`] bound or automatic version selection is used.
///
/// **日本語:**<br>
/// イベント表現を所有して受け取り、同じ Rust 型へ変換する、失敗し得る操作です。
/// 型の内部にバージョンを表現できますが、その検査や更新は実装が担います。
/// [`VersionedEvent`] の境界や自動的なバージョン選択は用いません。
pub trait EventUpcaster<Event> {
    /// **English:**<br>
    /// Transformation failure forwarded unchanged by [`UpcasterChain::apply`].
    ///
    /// **日本語:**<br>
    /// [`UpcasterChain::apply`] が変更せず伝播する変換の失敗です。
    type Error;

    /// **English:**<br>
    /// Consumes `event`, returning its transformed representation or an error.
    /// The input is not returned on error; side effects and panic conditions depend on the upcaster.
    ///
    /// **日本語:**<br>
    /// `event` を消費し、変換後の表現またはエラーを返します。
    /// エラー時に入力は返しません。副作用とパニックの条件は変換器に依存します。
    fn upcast(&self, event: Event) -> Result<Event, Self::Error>;
}

/// **English:**<br>
/// Insertion-ordered sequence of upcasters of a single concrete type.
/// Each stage consumes the preceding output. The chain does not select stages by event identity,
/// sort versions, or restore an event after failure. Use an enum or another common upcaster type
/// when stages need different behaviors. It stores no event value and adds no locking.
///
/// Add stages explicitly; each successful result becomes the next input.
///
/// **日本語:**<br>
/// 同じ具体型の変換器を追加順に保持する列です。
/// 各段階は直前の出力を消費します。イベント識別情報による段階の選択、バージョンの並べ替え、
/// 失敗後のイベント復元はしません。異なる動作の段階には列挙型などの共通の変換器型を用います。
/// イベント値は保持せず、ロックも追加しません。
///
/// 段階を明示的に追加すると、各段階の成功結果が次の入力になります。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_core::{EventUpcaster, UpcasterChain};
///
/// struct Append(&'static str);
/// impl EventUpcaster<String> for Append {
///     type Error = std::convert::Infallible;
///     fn upcast(&self, mut event: String) -> Result<String, Self::Error> {
///         event.push_str(self.0);
///         Ok(event)
///     }
/// }
/// let mut chain = UpcasterChain::new();
/// chain.push(Append("-v2"));
/// chain.push(Append("-v3"));
/// assert_eq!(chain.apply(String::from("v1")).unwrap(), "v1-v2-v3");
/// assert_eq!(chain.len(), 2);
/// ```
pub struct UpcasterChain<Event, Upcaster> {
    upcasters: Vec<Upcaster>,
    marker: PhantomData<fn() -> Event>,
}

impl<Event, Upcaster> Default for UpcasterChain<Event, Upcaster> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Event, Upcaster> UpcasterChain<Event, Upcaster> {
    /// **English:**<br>
    /// Creates an empty chain, also used by [`Default::default`].
    /// Applying an empty chain returns the input unchanged.
    ///
    /// **日本語:**<br>
    /// [`Default::default`] でも使う空の列を作ります。
    /// 空の列を適用すると、入力を変更せず返します。
    pub fn new() -> Self {
        Self {
            upcasters: Vec::new(),
            marker: PhantomData,
        }
    }

    /// **English:**<br>
    /// Appends `upcaster` as the last stage without inspecting its version or running it.
    ///
    /// **日本語:**<br>
    /// バージョンを調べたり実行したりせず、`upcaster` を最後の段階に追加します。
    pub fn push(&mut self, upcaster: Upcaster) {
        self.upcasters.push(upcaster);
    }

    /// **English:**<br>
    /// Returns the number of stored stages, not the number of supported event versions.
    ///
    /// **日本語:**<br>
    /// 対応するイベントバージョン数ではなく、保持する段階数を返します。
    pub fn len(&self) -> usize {
        self.upcasters.len()
    }

    /// **English:**<br>
    /// Returns `true` when no upcasters have been added.
    ///
    /// **日本語:**<br>
    /// 変換器を一つも追加していない場合に `true` を返します。
    pub fn is_empty(&self) -> bool {
        self.upcasters.is_empty()
    }

    /// **English:**<br>
    /// Runs all stages in insertion order and returns the final owned event.
    /// The first error stops evaluation and is returned unchanged; later stages are not called.
    /// Earlier side effects are not rolled back. An empty chain returns `Ok(event)`.
    /// Panics from a stage propagate, and the chain itself remains stored for future calls.
    ///
    /// **日本語:**<br>
    /// 全段階を追加順に実行し、最後の所有されたイベントを返します。
    /// 最初のエラーで評価を止め、変更せず返します。後続の段階は呼び出しません。
    /// 先行する副作用は取り消しません。空の列は `Ok(event)` を返します。
    /// 段階のパニックは伝播し、列自体は後続の呼び出しにも使えるよう保持されます。
    pub fn apply(&self, mut event: Event) -> Result<Event, Upcaster::Error>
    where
        Upcaster: EventUpcaster<Event>,
    {
        for upcaster in &self.upcasters {
            event = upcaster.upcast(event)?;
        }
        Ok(event)
    }
}
