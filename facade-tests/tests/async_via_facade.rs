// Immediate futures keep this adapter test runtime-independent and bounded:
// unexpected Pending fails instead of busy-spinning indefinitely.
mod support;

use ddd_toolkit::{
    AggregateRoot as _, AsyncDomainEventPublisher, AsyncRepository, Repository,
    dispatch_events_async,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::{Pin, pin},
    task::{Context, Poll, Waker},
};
use support::Store;

type PortFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Infallible>> + Send + 'a>>;

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test adapter unexpectedly suspended"),
    }
}

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct Placed;

#[derive(Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Placed>,
}

struct AsyncStore(Store<Order>);
impl AsyncRepository for AsyncStore {
    type Aggregate = Order;
    type Error = Infallible;
    fn find_by_id<'a>(&'a self, id: &'a u64) -> PortFuture<'a, Option<Order>> {
        Box::pin(async move { self.0.find_by_id(id) })
    }
    fn save<'a>(&'a mut self, aggregate: &'a Order) -> PortFuture<'a, ()> {
        Box::pin(async move { self.0.save(aggregate) })
    }
    fn delete<'a>(&'a mut self, id: &'a u64) -> PortFuture<'a, ()> {
        Box::pin(async move { self.0.delete(id) })
    }
    fn exists<'a>(&'a self, id: &'a u64) -> PortFuture<'a, bool> {
        Box::pin(async move { self.0.exists(id) })
    }
}

struct Publisher {
    accepted: Vec<Placed>,
    fail: bool,
}
impl AsyncDomainEventPublisher for Publisher {
    type Event = Placed;
    type Error = &'static str;
    fn publish<'a>(
        &'a mut self,
        event: &'a Placed,
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            if self.fail {
                return Err("broker down");
            }
            self.accepted.push(event.clone());
            Ok(())
        })
    }
}

#[test]
fn async_repository_and_dispatch_via_facade() {
    let mut store = AsyncStore(Store::new());
    let repository: &mut dyn AsyncRepository<Aggregate = Order, Error = Infallible> = &mut store;
    let mut order = Order {
        id: 1,
        events: vec![Placed],
    };
    ready(repository.save(&order)).unwrap();
    assert!(ready(repository.exists(&1)).unwrap());
    assert_eq!(ready(repository.find_by_id(&1)).unwrap().unwrap().id, 1);
    let mut publisher = Publisher {
        accepted: vec![],
        fail: true,
    };
    assert_eq!(
        ready(dispatch_events_async(&mut order, &mut publisher)),
        Err("broker down")
    );
    assert_eq!(order.events, vec![Placed]);
    publisher.fail = false;
    ready(dispatch_events_async(&mut order, &mut publisher)).unwrap();
    assert!(order.pull_events().is_empty());
    assert_eq!(publisher.accepted, vec![Placed]);
    ready(repository.delete(&1)).unwrap();
    assert!(ready(repository.find_by_id(&1)).unwrap().is_none());
}
