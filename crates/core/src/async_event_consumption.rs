#![allow(clippy::type_complexity)]

use crate::DomainEvent;
use std::future::Future;
use std::pin::Pin;

/// **English:**<br>
/// Handles borrowed domain events through boxed, `Send` futures.
///
/// Each future borrows the consumer mutably and the event for `'a`. No executor, retries,
/// or deduplication are supplied. Implementations define side effects and panic conditions;
/// dropping a pending future does not promise rollback of work already performed.
///
/// **日本語:**<br>
/// ボックス化された `Send` Future を通じて借用イベントを処理します。
///
/// 各 Future はコンシューマーを可変借用し、イベントを `'a` の間借用します。
/// 実行器、再試行、重複排除は提供しません。副作用とパニック条件は実装が定め、
/// 待機中の Future の破棄は、実行済み処理のロールバックを保証しません。
pub trait AsyncEventConsumer {
    /// **English:**<br>
    /// Event that can be shared by reference across threads.
    ///
    /// **日本語:**<br>
    /// スレッド間で参照を共有できるイベントです。
    type Event: DomainEvent + Sync;
    /// **English:**<br>
    /// Consumer failure that can be transferred across threads.
    ///
    /// **日本語:**<br>
    /// スレッド間で受け渡せるコンシューマーの失敗です。
    type Error: Send;

    /// **English:**<br>
    /// Returns a future resolving to completion or a handler error for `event`.
    /// Errors do not establish whether side effects occurred; retry policy belongs to the caller.
    ///
    /// **日本語:**<br>
    /// `event` の処理完了またはハンドラーのエラーを返す Future を作ります。
    /// エラーだけでは副作用の有無は判断できず、再試行方針は呼び出し側が定めます。
    fn consume<'a>(
        &'a mut self,
        event: &'a Self::Event,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'a>>;
}

/// **English:**<br>
/// Updates a read model through boxed, `Send` futures.
///
/// The future holds a mutable projection borrow and a shared event borrow for `'a`.
/// Implementations define persistence, ordering, duplicate handling, panics, and cancellation
/// behavior; this trait adds no scheduling or locking.
///
/// **日本語:**<br>
/// ボックス化された `Send` Future を通じて読み取りモデルを更新します。
///
/// Future はプロジェクションの可変借用とイベントの共有借用を `'a` の間保持します。
/// 永続化、順序、重複の扱い、パニック、キャンセル時の挙動は実装が定めます。
/// このトレイトはスケジューリングやロックを追加しません。
pub trait AsyncProjection {
    /// **English:**<br>
    /// Read-model input event with thread-shareable references.
    ///
    /// **日本語:**<br>
    /// スレッド間で参照を共有できる読み取りモデルの入力イベントです。
    type Event: DomainEvent + Sync;
    /// **English:**<br>
    /// Projection update error transferable across threads.
    ///
    /// **日本語:**<br>
    /// スレッド間で受け渡せるプロジェクション更新エラーです。
    type Error: Send;

    /// **English:**<br>
    /// Returns a future applying `event` and reporting completion or an update error.
    /// Neither an error nor dropping the future guarantees that partial updates were undone.
    ///
    /// **日本語:**<br>
    /// `event` を適用し、完了または更新エラーを報告する Future を返します。
    /// エラーや Future の破棄は、部分的な更新の取り消しを保証しません。
    fn project<'a>(
        &'a mut self,
        event: &'a Self::Event,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'a>>;
}
