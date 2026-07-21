//! A use case coordinating two separate aggregates through only
//! `ddd_toolkit::` paths: placing an `Order` reserves stock on a separate
//! `Inventory` aggregate, then dispatches the `OrderPlaced` event. This
//! covers three things the single-aggregate `domain_flow.rs` example
//! doesn't: a guard that runs before either aggregate is touched, two
//! aggregates persisted in the same use case, and a `DispatchError` that
//! surfaces *after* both aggregates are already saved - showing that a
//! failed dispatch doesn't roll back persistence, only leaves the caller
//! holding the undelivered events to retry.

use std::{
    future::Future,
    pin::pin,
    sync::Mutex,
    task::{Context, Poll, Waker},
};

use ddd_toolkit::{
    domain::{AggregateRoot, DomainEvent, Entity},
    mock::repository::InMemoryStore,
    port::PortError,
    port::event::{DispatchError, EventDispatcher},
    port::repository::{Load, Save},
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

#[derive(ddd_toolkit::EntityId, Clone, PartialEq, Debug)]
struct SkuId(String);

#[derive(Debug, Clone, PartialEq)]
struct OrderPlaced {
    order_id: String,
    sku: String,
    quantity: u32,
}

impl DomainEvent for OrderPlaced {}

#[derive(Debug, Clone)]
struct Order {
    id: OrderId,
    events: Vec<OrderPlaced>,
}

impl Order {
    fn place(id: &str, sku: &str, quantity: u32) -> Self {
        let mut order = Self {
            id: OrderId(id.to_string()),
            events: Vec::new(),
        };
        order.record(OrderPlaced {
            order_id: id.to_string(),
            sku: sku.to_string(),
            quantity,
        });
        order
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

/// Not surfaced as a domain event in this example - `Inventory` only reacts
/// to `OrderPlaced`, it doesn't publish anything of its own.
#[derive(Debug, Clone, PartialEq)]
struct NoInventoryEvent;

impl DomainEvent for NoInventoryEvent {}

#[derive(Debug, Clone)]
struct Inventory {
    id: SkuId,
    stock: u32,
}

impl Inventory {
    fn new(sku: &str, stock: u32) -> Self {
        Self {
            id: SkuId(sku.to_string()),
            stock,
        }
    }

    fn reserve(&mut self, quantity: u32) -> Result<(), u32> {
        if quantity > self.stock {
            return Err(self.stock);
        }
        self.stock -= quantity;
        Ok(())
    }
}

impl Entity for Inventory {
    type Id = SkuId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Inventory {
    type Event = NoInventoryEvent;

    fn record(&mut self, _event: Self::Event) {}

    fn take_events(&mut self) -> Vec<Self::Event> {
        Vec::new()
    }
}

#[derive(Debug)]
enum PlaceOrderError {
    InsufficientStock {
        sku: String,
        requested: u32,
        available: u32,
    },
    DispatchFailed(DispatchError<OrderPlaced>),
}

/// Reserves stock on `Inventory` first (touching nothing if that fails),
/// then persists `Order` and the reserved `Inventory`, then dispatches the
/// `OrderPlaced` event produced along the way.
async fn place_order<D: EventDispatcher<OrderPlaced>>(
    orders: &InMemoryStore<Order>,
    inventory: &InMemoryStore<Inventory>,
    dispatcher: &D,
    order_id: &str,
    sku: &str,
    quantity: u32,
) -> Result<(), PlaceOrderError> {
    let mut stock = inventory
        .load(&SkuId(sku.to_string()))
        .await
        .expect("inventory load should succeed")
        .unwrap_or_else(|| Inventory::new(sku, 0));

    stock
        .reserve(quantity)
        .map_err(|available| PlaceOrderError::InsufficientStock {
            sku: sku.to_string(),
            requested: quantity,
            available,
        })?;

    let mut order = Order::place(order_id, sku, quantity);
    orders
        .save(&mut order)
        .await
        .expect("order save should succeed");
    inventory
        .save(&mut stock)
        .await
        .expect("inventory save should succeed");

    let events = orders.take_recorded_events();
    dispatcher
        .dispatch(events)
        .await
        .map_err(PlaceOrderError::DispatchFailed)
}

struct RecordingDispatcher {
    dispatched: Mutex<Vec<OrderPlaced>>,
}

impl RecordingDispatcher {
    fn new() -> Self {
        Self {
            dispatched: Mutex::new(Vec::new()),
        }
    }
}

impl EventDispatcher<OrderPlaced> for RecordingDispatcher {
    async fn dispatch(&self, events: Vec<OrderPlaced>) -> Result<(), DispatchError<OrderPlaced>> {
        self.dispatched.lock().unwrap().extend(events);
        Ok(())
    }
}

struct AlwaysFailingDispatcher;

impl EventDispatcher<OrderPlaced> for AlwaysFailingDispatcher {
    async fn dispatch(&self, events: Vec<OrderPlaced>) -> Result<(), DispatchError<OrderPlaced>> {
        Err(DispatchError::new(
            events,
            PortError::unavailable("broker down"),
        ))
    }
}

#[test]
fn order_placement_reserves_inventory_and_dispatches_event_via_facade() {
    let orders = InMemoryStore::new();
    let inventory = InMemoryStore::new();
    block_on(inventory.save(&mut Inventory::new("widget", 10))).expect("seed save should succeed");
    let dispatcher = RecordingDispatcher::new();

    block_on(place_order(
        &orders,
        &inventory,
        &dispatcher,
        "order-1",
        "widget",
        3,
    ))
    .expect("place_order should succeed");

    let loaded_order = block_on(orders.load(&OrderId("order-1".to_string())))
        .expect("order load should succeed")
        .expect("order should have been saved");
    assert_eq!(loaded_order.id, OrderId("order-1".to_string()));

    let loaded_stock = block_on(inventory.load(&SkuId("widget".to_string())))
        .expect("inventory load should succeed")
        .expect("inventory should have been saved");
    assert_eq!(loaded_stock.stock, 7);

    assert_eq!(
        *dispatcher.dispatched.lock().unwrap(),
        vec![OrderPlaced {
            order_id: "order-1".to_string(),
            sku: "widget".to_string(),
            quantity: 3,
        }]
    );
}

#[test]
fn insufficient_stock_blocks_order_before_any_persistence_via_facade() {
    let orders = InMemoryStore::new();
    let inventory = InMemoryStore::new();
    block_on(inventory.save(&mut Inventory::new("widget", 2))).expect("seed save should succeed");
    let dispatcher = RecordingDispatcher::new();

    let error = block_on(place_order(
        &orders,
        &inventory,
        &dispatcher,
        "order-1",
        "widget",
        5,
    ))
    .expect_err("place_order should fail on insufficient stock");

    match error {
        PlaceOrderError::InsufficientStock {
            sku,
            requested,
            available,
        } => {
            assert_eq!(sku, "widget");
            assert_eq!(requested, 5);
            assert_eq!(available, 2);
        }
        PlaceOrderError::DispatchFailed(_) => panic!("expected InsufficientStock"),
    }

    assert!(
        block_on(orders.load(&OrderId("order-1".to_string())))
            .expect("order load should succeed")
            .is_none()
    );
    let loaded_stock = block_on(inventory.load(&SkuId("widget".to_string())))
        .expect("inventory load should succeed")
        .expect("seed inventory should still be present");
    assert_eq!(loaded_stock.stock, 2);
    assert!(dispatcher.dispatched.lock().unwrap().is_empty());
}

#[test]
fn dispatch_failure_after_persistence_surfaces_undelivered_order_event_via_facade() {
    let orders = InMemoryStore::new();
    let inventory = InMemoryStore::new();
    block_on(inventory.save(&mut Inventory::new("widget", 10))).expect("seed save should succeed");
    let dispatcher = AlwaysFailingDispatcher;

    let error = block_on(place_order(
        &orders,
        &inventory,
        &dispatcher,
        "order-1",
        "widget",
        3,
    ))
    .expect_err("place_order should fail when dispatch fails");

    let PlaceOrderError::DispatchFailed(dispatch_error) = error else {
        panic!("expected DispatchFailed");
    };
    assert_eq!(
        dispatch_error.undelivered,
        vec![OrderPlaced {
            order_id: "order-1".to_string(),
            sku: "widget".to_string(),
            quantity: 3,
        }]
    );

    // both aggregates were already committed - only propagation failed, so
    // the caller can retry dispatch from `dispatch_error.undelivered`
    // instead of redoing the whole use case.
    assert!(
        block_on(orders.load(&OrderId("order-1".to_string())))
            .expect("order load should succeed")
            .is_some()
    );
    let loaded_stock = block_on(inventory.load(&SkuId("widget".to_string())))
        .expect("inventory load should succeed")
        .expect("inventory should have been saved");
    assert_eq!(loaded_stock.stock, 7);
}
