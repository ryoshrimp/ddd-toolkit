use crate::event_restoration_guard::EventRestorationGuard;
use crate::{AggregateRoot, AsyncDomainEventPublisher};
use std::future::Future;
use std::pin::Pin;

/// **English:**<br>
/// Asynchronously publishes the events currently held by `aggregate`, in the order
/// returned by [`AggregateRoot::pull_events`]. On success, the events are removed. If publishing
/// fails, all pulled events are recorded back into the aggregate before the publisher error is
/// returned.
///
/// If [`AsyncDomainEventPublisher::publish`] or its returned future panics and unwinds, a Drop
/// guard records all pulled events back into the aggregate before unwinding continues. The guard
/// also restores all pulled events when the dispatch future is dropped after events have been
/// pulled but before dispatch completes. Dropping the dispatch future without polling it does not
/// pull any events.
///
/// Restoration does not undo external side effects of publishing, including publishes that already
/// succeeded; retrying can publish duplicates. Restoration may fail if destructors do not run
/// (for example, with `panic=abort`) or if [`AggregateRoot::record_event`] itself panics during
/// restoration. The aggregate's [`AggregateRoot::pull_events`] or [`AggregateRoot::record_event`]
/// implementation may also panic.
///
/// **日本語:**<br>
/// `aggregate` が保持するイベントを [`AggregateRoot::pull_events`] の返す順序で非同期に
/// 発行します。成功時はイベントを取り除きます。発行に失敗した場合、パブリッシャーのエラーを
/// 返す前に取り出した全イベントを集約へ記録し直します。
///
/// [`AsyncDomainEventPublisher::publish`] またはそれが返すfutureのパニックがアンワインド
/// する場合、Dropガードが取り出した全イベントを集約へ記録し直してから、アンワインドが
/// 継続します。イベント取り出し後、ディスパッチ完了前にディスパッチのfutureがdropされた
/// 場合も、ガードが取り出した全イベントを復元します。ディスパッチのfutureが一度もpoll
/// されなければ、イベントは取り出されません。
///
/// 復元は、すでに成功した発行を含め、発行による外部副作用を取り消しません。再試行すると
/// 重複発行される可能性があります。`panic=abort` などでデストラクタが実行されない場合や、
/// 復元中に [`AggregateRoot::record_event`] 自体がパニックする場合、復元は保証されません。
/// 集約の [`AggregateRoot::pull_events`] または [`AggregateRoot::record_event`] の実装も
/// パニックする可能性があります。
pub fn dispatch_events_async<'a, A, P>(
    aggregate: &'a mut A,
    publisher: &'a mut P,
) -> Pin<Box<dyn Future<Output = Result<(), P::Error>> + Send + 'a>>
where
    A: AggregateRoot + Send + 'a,
    A::Event: Send + Sync + 'a,
    P: AsyncDomainEventPublisher<Event = A::Event> + Send + 'a,
    P::Error: 'a,
{
    Box::pin(async move {
        let snapshot = aggregate.pull_events();
        let mut guard = EventRestorationGuard::new(aggregate, snapshot);

        for event in guard.events() {
            publisher.publish(event).await?
        }

        guard.commit();
        Ok(())
    })
}
