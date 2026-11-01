/// **English:**<br>
/// Owned next state and ordered effect descriptions returned by a [`ProcessManager`].
/// Construction and access do not execute effects or persist the state. Empty effects are valid;
/// callers choose how to coordinate state storage and effect execution.
///
/// Inspect the transition before taking ownership of both components.
///
/// **日本語:**<br>
/// [`ProcessManager`] が返す、所有された次状態と順序付きの副作用の記述です。
/// 構築や参照で副作用を実行したり状態を永続化したりしません。空の副作用列も許容し、
/// 状態保存と副作用実行の連携方法は呼び出し側が決めます。
///
/// 遷移を参照してから、両方の構成要素の所有権を取り出します。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_core::Transition;
///
/// let transition = Transition::new("paid", vec!["send receipt"]);
/// assert_eq!(transition.state(), &"paid");
/// assert_eq!(transition.effects(), &["send receipt"]);
/// let (state, effects) = transition.into_parts();
/// assert_eq!((state, effects), ("paid", vec!["send receipt"]));
/// ```
pub struct Transition<State, Effect> {
    state: State,
    effects: Vec<Effect>,
}

impl<State, Effect> Transition<State, Effect> {
    /// **English:**<br>
    /// Stores `state` and `effects` unchanged, preserving the vector's order without validation.
    ///
    /// **日本語:**<br>
    /// 検証せず、ベクターの順序を保ったまま `state` と `effects` を保持します。
    pub fn new(state: State, effects: Vec<Effect>) -> Self {
        Self { state, effects }
    }

    /// **English:**<br>
    /// Borrows the next state without applying it to any external store.
    ///
    /// **日本語:**<br>
    /// 外部ストアへの反映を行わず、次状態を借用します。
    pub fn state(&self) -> &State {
        &self.state
    }

    /// **English:**<br>
    /// Borrows the effect descriptions in their stored order without executing them.
    ///
    /// **日本語:**<br>
    /// 実行せず、保持している順序で副作用の記述を借用します。
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    /// **English:**<br>
    /// Consumes the transition to return its state and effect vector without cloning.
    ///
    /// **日本語:**<br>
    /// 遷移を消費し、複製せずに状態と副作用のベクターを返します。
    pub fn into_parts(self) -> (State, Vec<Effect>) {
        (self.state, self.effects)
    }
}

/// **English:**<br>
/// Computes a next state and effect descriptions from borrowed state and event values.
/// The caller owns persistence, event delivery, and effect execution. The signature does not enforce
/// determinism, reentrancy, idempotency, or purity; shared access can still use interior mutability.
/// Thread-safety bounds and panic behavior are left to implementations.
///
/// **日本語:**<br>
/// 借用した状態とイベントから、次状態と副作用の記述を算出します。
/// 永続化、イベント配信、副作用の実行は呼び出し側が担います。このシグネチャは決定性、再入可能性、
/// 冪等性、純粋性を強制せず、共有参照でも内部可変性を使えます。
/// スレッド安全性の境界とパニックの条件は実装が定めます。
pub trait ProcessManager {
    /// **English:**<br>
    /// Process state borrowed as input and owned by the resulting transition.
    ///
    /// **日本語:**<br>
    /// 入力として借用し、結果の遷移が所有するプロセス状態です。
    type State;

    /// **English:**<br>
    /// Event type interpreted by this manager; no event trait bound is required.
    ///
    /// **日本語:**<br>
    /// このマネージャーが解釈するイベント型です。イベントトレイトの境界は要求しません。
    type Event;

    /// **English:**<br>
    /// Description of an action for the caller to execute after handling.
    ///
    /// **日本語:**<br>
    /// 処理後に呼び出し側が実行する動作の記述です。
    type Effect;

    /// **English:**<br>
    /// Handling failure returned instead of a transition.
    ///
    /// **日本語:**<br>
    /// 遷移の代わりに返す処理の失敗です。
    type Error;

    /// **English:**<br>
    /// Evaluates `event` against `state`, returning a transition or a handling error.
    /// Neither input is consumed. On error no transition is returned, but the trait does not
    /// undo implementation side effects or guarantee that no side effects occurred.
    ///
    /// **日本語:**<br>
    /// `state` に対して `event` を評価し、遷移または処理エラーを返します。
    /// どちらの入力も消費しません。エラー時は遷移を返しませんが、実装の副作用を取り消したり、
    /// 副作用が発生していないと保証したりはしません。
    fn handle(
        &self,
        state: &Self::State,
        event: &Self::Event,
    ) -> Result<Transition<Self::State, Self::Effect>, Self::Error>;
}
