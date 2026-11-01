//! Advanced: retry a credit notification without increasing the balance twice.
//! 発展編: 入金通知を再試行しても残高を二重加算しない。async/Futureの基礎が前提。
//!
//! Run: cargo run -p ddd-toolkit --example reliable_delivery --locked
//! Read main → deliver → adapter implementations. Skip ready's polling details first.
//! main → deliver → アダプターの順に読む。最初はreadyのpollの詳細を飛ばしてよい。
//! Outbox = saved pending notifications; projection = event-derived read state;
//! checkpoint = processing position; stable IDs deduplicate repeated application.
//! outbox=送信待ちの記録、projection=イベントから作る参照状態、checkpoint=処理位置。
//! Expected: broker/checkpoint/ack failures, three deliveries, balance still 10.
//! ブローカー・位置保存・ack失敗から再試行し、配信3回でも残高は10のまま。
//! Memory adapters demonstrate contracts, not durable or exactly-once delivery.
//! メモリ実装は契約の教材。永続性やexactly-once配信を提供するものではない。
use ddd_toolkit::{
    AggregateRoot as _, AppendError, AppendResult, AsyncDomainEventPublisher,
    AsyncIdempotentProjection, AsyncProjectionCheckpointStore, AsyncTransactionalOutbox,
    AsyncTransactionalOutboxLifecycleContext, CurrentRevision, ExpectedRevision, NoStream,
    OutboxClaim, OutboxMessageId, ProjectionApplication, ProjectionCheckpoint, RevisionConflict,
    StreamRevision,
};
use std::{
    collections::{HashSet, VecDeque},
    future::Future,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};

type PortFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, &'static str>> + Send + 'a>>;

// Not an executor: only these non-suspending demo adapters can use this helper.
// Real I/O adapters need an application-selected runtime, wakeups and cancellation handling.
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("demo adapter unexpectedly suspended; use a real executor for I/O"),
    }
}
#[derive(Debug, Clone, ddd_toolkit::DomainEvent)]
struct Credited(u32);
#[derive(Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Account {
    #[entity(id)]
    id: u64,
    balance: u32,
    #[aggregate(events)]
    events: Vec<Credited>,
}

// Single-account adapter. The queue head is reclaimed until acknowledged.
// A production adapter must atomically persist state + queue, and define claim leases.
#[derive(Default)]
struct Outbox {
    saved: Option<Account>,
    revision: Option<StreamRevision>,
    queue: VecDeque<(OutboxMessageId, Credited)>,
    fail_ack: bool,
}
impl AsyncTransactionalOutbox for Outbox {
    type Aggregate = Account;
    type Error = &'static str;
    fn save_and_enqueue<'a>(
        &'a mut self,
        account: &'a Account,
        expected: ExpectedRevision,
        events: Vec<Credited>,
    ) -> Pin<Box<dyn Future<Output = Result<AppendResult, AppendError<Self::Error>>> + Send + 'a>>
    {
        Box::pin(async move {
            if events.is_empty() {
                return Err(AppendError::EmptyBatch);
            }
            if self
                .saved
                .as_ref()
                .is_some_and(|saved| saved.id != account.id)
            {
                return Err(AppendError::Adapter("demo stores only one account"));
            }
            let matches = match (expected, self.revision) {
                (ExpectedRevision::NoStream(_), None) => true,
                (ExpectedRevision::Revision(want), Some(have)) => want == have,
                _ => false,
            };
            if !matches {
                return Err(AppendError::Conflict(RevisionConflict {
                    expected,
                    current: self.revision.map_or(
                        CurrentRevision::NoStream(NoStream),
                        CurrentRevision::Revision,
                    ),
                }));
            }
            let first = match self.revision {
                None => 0,
                Some(current) => current
                    .value()
                    .checked_add(1)
                    .ok_or(AppendError::RevisionExhausted { current })?,
            };
            let last = first.checked_add(events.len() as u64 - 1).ok_or(
                AppendError::RevisionExhausted {
                    current: StreamRevision::new(first),
                },
            )?;
            let mut snapshot = account.clone();
            snapshot.pull_events();
            self.saved = Some(snapshot);
            self.revision = Some(StreamRevision::new(last));
            for (offset, event) in events.into_iter().enumerate() {
                self.queue.push_back((
                    OutboxMessageId::new(format!("{}:{}", account.id, first + offset as u64)),
                    event,
                ));
            }
            Ok(AppendResult {
                resulting_revision: StreamRevision::new(last),
            })
        })
    }
    fn claim<'a>(&'a mut self) -> PortFuture<'a, Option<OutboxClaim<Credited>>> {
        Box::pin(async move {
            Ok(self
                .queue
                .front()
                .map(|(id, event)| OutboxClaim::new(id.clone(), event.clone())))
        })
    }
    fn acknowledge<'a>(&'a mut self, id: &'a OutboxMessageId) -> PortFuture<'a, ()> {
        Box::pin(async move {
            if self.fail_ack {
                return Err("acknowledgement lost");
            }
            if self.queue.front().is_none_or(|(head, _)| head != id) {
                return Err("unknown claim");
            }
            self.queue.pop_front();
            Ok(())
        })
    }
}

#[derive(Default)]
struct Broker {
    fail: bool,
    accepted: Vec<Credited>,
}
impl AsyncDomainEventPublisher for Broker {
    type Event = Credited;
    type Error = &'static str;
    fn publish<'a>(&'a mut self, event: &'a Credited) -> PortFuture<'a, ()> {
        Box::pin(async move {
            if self.fail {
                return Err("broker down");
            }
            self.accepted.push(event.clone());
            Ok(())
        })
    }
}
#[derive(Default)]
struct ReadModel {
    balance: u32,
    applied: HashSet<String>,
}
impl AsyncIdempotentProjection for ReadModel {
    type Event = Credited;
    type Cursor = String;
    type Error = &'static str;
    fn apply_once<'a>(
        &'a mut self,
        event: &'a Credited,
        checkpoint: &'a ProjectionCheckpoint<String>,
    ) -> PortFuture<'a, ProjectionApplication> {
        Box::pin(async move {
            if self.applied.contains(checkpoint.cursor()) {
                return Ok(ProjectionApplication::AlreadyApplied);
            }
            let balance = self
                .balance
                .checked_add(event.0)
                .ok_or("balance overflow")?;
            // Deduplication marker + read model update must be atomic in production too.
            self.applied.insert(checkpoint.cursor().clone());
            self.balance = balance;
            Ok(ProjectionApplication::Applied)
        })
    }
}
#[derive(Default)]
struct Checkpoints {
    last: Option<ProjectionCheckpoint<String>>,
    fail_save: bool,
}
impl AsyncProjectionCheckpointStore for Checkpoints {
    type Cursor = String;
    type Error = &'static str;
    fn load<'a>(&'a self) -> PortFuture<'a, Option<ProjectionCheckpoint<String>>> {
        Box::pin(async move { Ok(self.last.clone()) })
    }
    fn save<'a>(&'a mut self, checkpoint: ProjectionCheckpoint<String>) -> PortFuture<'a, ()> {
        Box::pin(async move {
            if self.fail_save {
                return Err("checkpoint unavailable");
            }
            self.last = Some(checkpoint);
            Ok(())
        })
    }
}

async fn deliver(
    outbox: &mut Outbox,
    broker: &mut Broker,
    model: &mut ReadModel,
    checkpoints: &mut Checkpoints,
) -> Result<ProjectionApplication, &'static str> {
    let claim = outbox.claim().await?.ok_or("queue empty")?;
    let (id, event) = claim.into_parts();
    broker.publish(&event).await?;
    // This demo directly feeds a consumer after publishing; no real broker is used.
    // Message IDs are stable across retries; these cursors are not sortable offsets.
    let checkpoint = ProjectionCheckpoint::new(id.as_str().to_owned());
    let result = model.apply_once(&event, &checkpoint).await?;
    checkpoints.save(checkpoint).await?;
    outbox.acknowledge(&id).await?;
    Ok(result)
}
fn main() {
    let mut account = Account {
        id: 1,
        balance: 10,
        events: vec![Credited(10)],
    };
    let mut outbox = Outbox::default();
    let mut context = AsyncTransactionalOutboxLifecycleContext::new(
        &mut account,
        &mut outbox,
        ExpectedRevision::NoStream(NoStream),
    );
    ready(context.save_and_enqueue()).unwrap();
    context.complete().unwrap();
    assert!(account.pull_events().is_empty());
    assert_eq!(outbox.saved.as_ref().unwrap().balance, 10);
    let mut broker = Broker {
        fail: true,
        ..Broker::default()
    };
    let mut model = ReadModel::default();
    let mut checkpoints = Checkpoints::default();
    assert_eq!(
        ready(deliver(
            &mut outbox,
            &mut broker,
            &mut model,
            &mut checkpoints
        )),
        Err("broker down")
    );
    assert_eq!(outbox.queue.len(), 1);
    assert_eq!(model.balance, 0);

    broker.fail = false;
    checkpoints.fail_save = true;
    assert_eq!(
        ready(deliver(
            &mut outbox,
            &mut broker,
            &mut model,
            &mut checkpoints
        )),
        Err("checkpoint unavailable")
    );
    assert_eq!(model.balance, 10);
    assert!(ready(checkpoints.load()).unwrap().is_none());
    checkpoints.fail_save = false;
    outbox.fail_ack = true;
    assert_eq!(
        ready(deliver(
            &mut outbox,
            &mut broker,
            &mut model,
            &mut checkpoints
        )),
        Err("acknowledgement lost")
    );
    assert_eq!(model.balance, 10);
    assert_eq!(outbox.queue.len(), 1);
    outbox.fail_ack = false;
    assert_eq!(
        ready(deliver(
            &mut outbox,
            &mut broker,
            &mut model,
            &mut checkpoints
        )),
        Ok(ProjectionApplication::AlreadyApplied)
    );
    assert_eq!(model.balance, 10);
    assert_eq!(broker.accepted.len(), 3); // At-least-once delivery, not exactly-once.
    assert_eq!(ready(checkpoints.load()).unwrap().unwrap().cursor(), "1:0");
    assert!(ready(outbox.claim()).unwrap().is_none());
    println!("reliable_delivery: retried broker/checkpoint/ack failures; 3 deliveries, balance=10");
}
