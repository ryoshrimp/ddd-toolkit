use crate::event_restoration_guard::EventRestorationGuard;
use crate::{AggregateRoot, DomainEventPublisher};

/// **English:**<br>
/// Publishes the events currently held by `aggregate` through `publisher`.
///
/// Events are passed to [`DomainEventPublisher::publish`] in the order returned by
/// [`AggregateRoot::pull_events`]. If a publish operation returns an error, all events pulled at
/// the start of this call are recorded back into the aggregate before that error is returned.
/// If every publish succeeds, the events are removed from the aggregate and the function returns
/// `Ok(())`.
///
/// If [`DomainEventPublisher::publish`] panics and unwinds, a Drop guard records all pulled events
/// back into the aggregate before unwinding continues. Restoration does not undo external side
/// effects of publishing, including publishes that already succeeded; retrying can publish
/// duplicates. Restoration may fail if destructors do not run (for example, with `panic=abort`)
/// or if [`AggregateRoot::record_event`] itself panics during restoration.
///
/// The function may panic if the aggregate's [`AggregateRoot::pull_events`] or
/// [`AggregateRoot::record_event`] implementation panics. It does not impose additional
/// synchronization; the aggregate and publisher must be mutably borrowed by the caller for the
/// duration of the dispatch.
///
/// **日本語:**<br>
/// `aggregate` が現在保持しているイベントを `publisher` 経由で発行します。
///
/// イベントは [`AggregateRoot::pull_events`] が返した順序で [`DomainEventPublisher::publish`]
/// に渡されます。発行処理がエラーを返すと、この呼び出しの開始時に取り出したすべての
/// イベントを集約へ記録し直してから、そのエラーを返します。すべての発行に成功した場合、
/// イベントは集約から削除され、`Ok(())` を返します。
///
/// [`DomainEventPublisher::publish`] のパニックがアンワインドする場合、Dropガードが
/// 取り出した全イベントを集約へ記録し直してから、アンワインドが継続します。復元は、
/// すでに成功した発行を含め、発行による外部副作用を取り消しません。再試行すると重複発行
/// される可能性があります。`panic=abort` などでデストラクタが実行されない場合や、
/// 復元中に [`AggregateRoot::record_event`] 自体がパニックする場合、復元は保証されません。
///
/// 集約の [`AggregateRoot::pull_events`] または [`AggregateRoot::record_event`] の実装が
/// パニックすると、この関数もパニックする可能性があります。この関数は追加の同期を
/// 行わないため、呼び出し側はディスパッチ中、集約とパブリッシャーを可変借用します。
pub fn dispatch_events<A, P>(aggregate: &mut A, publisher: &mut P) -> Result<(), P::Error>
where
    A: AggregateRoot,
    P: DomainEventPublisher<Event = A::Event>,
{
    let snapshot = aggregate.pull_events();
    let mut guard = EventRestorationGuard::new(aggregate, snapshot);

    for event in guard.events() {
        publisher.publish(event)?;
    }

    guard.commit();
    Ok(())
}
