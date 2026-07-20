//! A small but complete example wiring an aggregate, a repository, and an
//! event dispatcher together using only `ddd_toolkit::` paths - proves the
//! toolkit actually composes end-to-end for a real consumer, and exercises
//! the `DispatchError<E>` partial-failure path from the facade side.

use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use ddd_toolkit::{
    domain::{AggregateRoot, DomainEvent, Entity},
    mock::repository::InMemoryStore,
    port::PortError,
    port::event::{DispatchError, EventDispatcher},
    port::repository::{Delete, Load, Save},
};

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct OrderId(String);

#[derive(Debug, Clone, PartialEq)]
struct OrderPlaced {
    order_id: String,
}

impl DomainEvent for OrderPlaced {}

#[derive(Debug, Clone)]
struct Order {
    id: OrderId,
    total_cents: u32,
    events: Vec<OrderPlaced>,
}

impl Order {
    fn new(id: &str, total_cents: u32) -> Self {
        Self {
            id: OrderId(id.to_string()),
            total_cents,
            events: Vec::new(),
        }
    }
}

impl Entity for Order {
    type Id = OrderId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Order {
    type Event = OrderPlaced;

    fn record(&mut self, event: Self::Event) {
        self.events.push(event);
    }

    fn take_events(&mut self) -> Vec<Self::Event> {
        std::mem::take(&mut self.events)
    }
}

#[test]
fn order_round_trips_through_in_memory_store_via_facade() {
    let store = InMemoryStore::new();
    let mut order = Order::new("order-1", 4200);
    order.record(OrderPlaced {
        order_id: "order-1".to_string(),
    });

    block_on(store.save(&mut order)).expect("save should succeed");

    let loaded = block_on(store.load(&OrderId("order-1".to_string())))
        .expect("load should succeed")
        .expect("order should be found");
    assert_eq!(loaded.total_cents, 4200);

    block_on(store.delete(&OrderId("order-1".to_string()))).expect("delete should succeed");
    let after_delete =
        block_on(store.load(&OrderId("order-1".to_string()))).expect("load should succeed");
    assert!(after_delete.is_none());

    assert_eq!(
        store.take_recorded_events(),
        vec![OrderPlaced {
            order_id: "order-1".to_string()
        }]
    );
}

struct PartiallyFailingDispatcher;

impl EventDispatcher<OrderPlaced> for PartiallyFailingDispatcher {
    async fn dispatch(&self, events: Vec<OrderPlaced>) -> Result<(), DispatchError<OrderPlaced>> {
        Err(DispatchError::new(
            events,
            PortError::unavailable("broker down"),
        ))
    }
}

#[test]
fn event_dispatch_failure_reports_undelivered_events_via_facade() {
    let dispatcher = PartiallyFailingDispatcher;
    let events = vec![OrderPlaced {
        order_id: "order-1".to_string(),
    }];

    let error = block_on(dispatcher.dispatch(events)).expect_err("dispatch should fail");

    assert_eq!(
        error.undelivered,
        vec![OrderPlaced {
            order_id: "order-1".to_string()
        }]
    );
}
