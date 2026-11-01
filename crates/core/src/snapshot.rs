use crate::event_store::StreamRevision;
use crate::{AggregateRoot, Entity};

/// **English:**<br>
/// Owned aggregate state paired with the last event revision it represents.
/// No identifier or schema version is added; include validation metadata in `S` when needed.
///
/// **日本語:**<br>
/// 集約状態と、その状態が反映する最後のイベントのリビジョンを所有します。
/// 識別子やスキーマバージョンは追加しません。検証用メタデータは必要に応じて `S` に含めます。
///
/// # Example / 使用例
///
/// ```rust
/// use ddd_toolkit_core::{AggregateSnapshot, StreamRevision};
/// let snapshot = AggregateSnapshot::new(42_u64, StreamRevision::new(3));
/// assert_eq!(*snapshot.state(), 42);
/// assert_eq!(snapshot.into_parts(), (42, StreamRevision::new(3)));
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct AggregateSnapshot<S> {
    state: S,
    revision: StreamRevision,
}

impl<S> AggregateSnapshot<S> {
    /// **English:**<br>
    /// Pairs `state` with `revision` without checking consistency or compatibility.
    ///
    /// **日本語:**<br>
    /// 整合性や互換性を検証せず、`state` と `revision` を組み合わせます。
    pub fn new(state: S, revision: StreamRevision) -> Self {
        Self { state, revision }
    }

    /// **English:**<br>
    /// Borrows the stored aggregate state.
    ///
    /// **日本語:**<br>
    /// 保存された集約状態を借用します。
    pub fn state(&self) -> &S {
        &self.state
    }

    /// **English:**<br>
    /// Returns the last event revision represented by the state.
    ///
    /// **日本語:**<br>
    /// 状態が反映する最後のイベントのリビジョンを返します。
    pub fn revision(&self) -> StreamRevision {
        self.revision
    }

    /// **English:**<br>
    /// Consumes the snapshot, returning state and revision without cloning.
    ///
    /// **日本語:**<br>
    /// スナップショットを消費し、複製せずに状態とリビジョンを返します。
    pub fn into_parts(self) -> (S, StreamRevision) {
        (self.state, self.revision)
    }
}

/// **English:**<br>
/// Aggregate-defined snapshot classification used by [`crate::replay_with_optional_snapshot`].
/// Both `Stale` and `Inapplicable` trigger full replay; an older but usable snapshot may be `Applicable`.
///
/// **日本語:**<br>
/// [`crate::replay_with_optional_snapshot`] で使う集約定義のスナップショット分類です。
/// `Stale` と `Inapplicable` は全件再生を選択します。古くても利用可能なら `Applicable` にできます。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotUse {
    /// **English:**<br>
    /// Restore this snapshot and replay only later events.
    ///
    /// **日本語:**<br>
    /// このスナップショットを復元し、後続イベントのみ再生します。
    Applicable,
    /// **English:**<br>
    /// Reject this snapshot as stale and replay the complete history.
    ///
    /// **日本語:**<br>
    /// 古いスナップショットとして採用せず、完全な履歴を再生します。
    Stale,
    /// **English:**<br>
    /// Reject this snapshot as incompatible and replay the complete history.
    ///
    /// **日本語:**<br>
    /// 互換性のないスナップショットとして採用せず、完全な履歴を再生します。
    Inapplicable,
}

/// **English:**<br>
/// Snapshot lookup failure, distinct from a missing snapshot.
///
/// **日本語:**<br>
/// スナップショットの検索失敗です。スナップショットの不在とは区別されます。
#[derive(Debug, Eq, PartialEq)]
pub enum SnapshotLoadError<E> {
    /// **English:**<br>
    /// Error reported by the storage adapter.
    ///
    /// **日本語:**<br>
    /// ストレージアダプターが報告したエラーです。
    Adapter(E),
}

/// **English:**<br>
/// Failure to persist a snapshot.
///
/// **日本語:**<br>
/// スナップショットの永続化失敗です。
#[derive(Debug, Eq, PartialEq)]
pub enum SnapshotSaveError<E> {
    /// **English:**<br>
    /// Storage adapter error during saving.
    ///
    /// **日本語:**<br>
    /// 保存時のストレージアダプターエラーです。
    Adapter(E),
}

/// **English:**<br>
/// Synchronous storage boundary for snapshots indexed by aggregate identifier.
/// Implementations define replacement policy, durability, synchronization, and panic conditions.
/// Saving a snapshot does not itself append events or validate the snapshot against an event stream.
///
/// **日本語:**<br>
/// 集約識別子をキーとするスナップショットの同期ストレージ境界です。
/// 置換方針、永続性、同期、パニック条件は実装が定めます。
/// 保存自体はイベント追加やイベントストリームに対するスナップショットの検証を行いません。
pub trait SnapshotStore {
    /// **English:**<br>
    /// Aggregate defining the snapshot lookup identifier.
    ///
    /// **日本語:**<br>
    /// スナップショット検索の識別子を定める集約です。
    type Aggregate: AggregateRoot;
    /// **English:**<br>
    /// State representation stored in each snapshot.
    ///
    /// **日本語:**<br>
    /// 各スナップショットに保存する状態の表現です。
    type State;
    /// **English:**<br>
    /// Adapter-specific persistence error.
    ///
    /// **日本語:**<br>
    /// アダプター固有の永続化エラーです。
    type Error;

    /// **English:**<br>
    /// Loads a snapshot for `id`; `Ok(None)` means none is available.
    /// Storage failures use [`SnapshotLoadError::Adapter`] rather than the absence result.
    ///
    /// **日本語:**<br>
    /// `id` のスナップショットを読み込みます。`Ok(None)` は不在を表します。
    /// ストレージ障害は不在ではなく [`SnapshotLoadError::Adapter`] で返します。
    fn load(
        &self,
        id: &<Self::Aggregate as Entity>::Id,
    ) -> Result<Option<AggregateSnapshot<Self::State>>, SnapshotLoadError<Self::Error>>;

    /// **English:**<br>
    /// Consumes and saves `snapshot` for `id`, returning an adapter error on failure.
    /// The snapshot is not returned on error; replacement and retry behavior depend on the adapter.
    ///
    /// **日本語:**<br>
    /// `id` に対して `snapshot` を消費して保存し、失敗時はアダプターエラーを返します。
    /// エラー時にスナップショットは返されません。置換や再試行の挙動はアダプターに依存します。
    fn save(
        &mut self,
        id: &<Self::Aggregate as Entity>::Id,
        snapshot: AggregateSnapshot<Self::State>,
    ) -> Result<(), SnapshotSaveError<Self::Error>>;
}
