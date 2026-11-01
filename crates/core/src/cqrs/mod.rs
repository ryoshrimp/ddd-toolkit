//! **English:**<br>
//! CQRS handlers and lifecycle coordination for aggregate commands and queries.
//!
//! Lifecycle contexts require a successful save and event dispatch before completion.
//! A failed attempt is consumed and cannot be retried through the same context.
//!
//! **日本語:**<br>
//! 集約に対するコマンド・クエリの CQRS ハンドラーとライフサイクル調整機能を提供します。
//!
//! ライフサイクルコンテキストは完了前に保存とイベント配信の成功を要求します。
//! 失敗した操作は消費済みとなり、同じコンテキストから再試行できません。

mod async_query_handler;
mod query_handler;

pub use async_query_handler::*;

pub use query_handler::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// **English:**<br>
/// Records whether a lifecycle operation is pending, successful, or consumed by failure.
///
/// **日本語:**<br>
/// ライフサイクル操作が未実行、成功、または失敗により消費済みかを表します。
pub enum OperationState {
    /// **English:**<br>
    /// The operation has not been attempted.
    ///
    /// **日本語:**<br>
    /// 操作はまだ試行されていません。
    Pending,
    /// **English:**<br>
    /// The operation completed successfully.
    ///
    /// **日本語:**<br>
    /// 操作は正常に完了しました。
    Succeeded,
    /// **English:**<br>
    /// The operation failed or its future was dropped after starting; retry is disallowed.
    ///
    /// **日本語:**<br>
    /// 操作が失敗したか開始後に future が破棄され、再試行できません。
    FailedConsumed,
}

#[derive(Debug, PartialEq, Eq)]
/// **English:**<br>
/// An error returned when a lifecycle save or dispatch cannot be attempted or its adapter fails.
///
/// **日本語:**<br>
/// ライフサイクルの保存・配信を実行できない場合、またはアダプターが失敗した場合のエラーです。
pub enum LifecycleOperationError<E> {
    /// **English:**<br>
    /// The repository or publisher returned this adapter error.
    ///
    /// **日本語:**<br>
    /// リポジトリまたはパブリッシャーが返したアダプターエラーです。
    Adapter(E),
    /// **English:**<br>
    /// Save was already attempted on this context.
    ///
    /// **日本語:**<br>
    /// このコンテキストでは保存がすでに試行されています。
    AlreadySaved,
    /// **English:**<br>
    /// Dispatch was already attempted on this context.
    ///
    /// **日本語:**<br>
    /// このコンテキストでは配信がすでに試行されています。
    AlreadyDispatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// **English:**<br>
/// Describes why a lifecycle context cannot be completed.
///
/// **日本語:**<br>
/// ライフサイクルコンテキストを完了できない理由を示します。
pub enum LifecycleCompletionError {
    /// **English:**<br>
    /// No save was attempted before completion.
    ///
    /// **日本語:**<br>
    /// 完了前に保存が試行されていません。
    MissingSave,
    /// **English:**<br>
    /// No event dispatch was attempted before completion.
    ///
    /// **日本語:**<br>
    /// 完了前にイベント配信が試行されていません。
    MissingDispatch,
    /// **English:**<br>
    /// The save attempt failed and was consumed.
    ///
    /// **日本語:**<br>
    /// 保存の試行が失敗し、消費済みです。
    SaveFailedConsumed,
    /// **English:**<br>
    /// The dispatch attempt failed and was consumed.
    ///
    /// **日本語:**<br>
    /// 配信の試行が失敗し、消費済みです。
    DispatchFailedConsumed,
}
