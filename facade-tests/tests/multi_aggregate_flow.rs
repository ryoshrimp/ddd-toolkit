// Persistence and publishing are independent; these test-local adapters do not
// provide a transaction across aggregates or roll back saves on publish failure.
mod support;

use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, Repository, dispatch_events};
use support::Store;

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct Placed {
    order_id: u64,
    quantity: u32,
}

#[derive(Debug, Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Placed>,
}

#[derive(Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Inventory {
    #[entity(id)]
    id: u64,
    stock: u32,
    #[aggregate(events)]
    events: Vec<Placed>,
}

#[derive(Debug, PartialEq)]
enum PlaceError {
    InsufficientStock,
    PublishFailed(Order),
}

fn place_order(
    orders: &mut Store<Order>,
    inventory: &mut Store<Inventory>,
    publisher: &mut impl DomainEventPublisher<Event = Placed, Error = &'static str>,
    quantity: u32,
) -> Result<(), PlaceError> {
    let mut stock = inventory.find_by_id(&1).unwrap().unwrap();
    if stock.stock < quantity {
        return Err(PlaceError::InsufficientStock);
    }
    stock.stock -= quantity;
    let mut order = Order {
        id: 10,
        events: vec![],
    };
    order.record_event(Placed {
        order_id: 10,
        quantity,
    });
    orders.save(&order).unwrap();
    inventory.save(&stock).unwrap();
    dispatch_events(&mut order, publisher).map_err(|_| PlaceError::PublishFailed(order))
}

#[derive(Default)]
struct Publisher {
    accepted: Vec<Placed>,
    fail: bool,
}
impl DomainEventPublisher for Publisher {
    type Event = Placed;
    type Error = &'static str;
    fn publish(&mut self, event: &Placed) -> Result<(), Self::Error> {
        if self.fail {
            return Err("broker down");
        }
        self.accepted.push(event.clone());
        Ok(())
    }
}

fn inventory(stock: u32) -> Store<Inventory> {
    let mut store = Store::new();
    store
        .save(&Inventory {
            id: 1,
            stock,
            events: vec![],
        })
        .unwrap();
    store
}

#[test]
fn placement_reserves_stock_and_publishes() {
    let mut orders = Store::new();
    let mut inventory = inventory(10);
    let mut publisher = Publisher::default();
    place_order(&mut orders, &mut inventory, &mut publisher, 3).unwrap();
    assert!(orders.exists(&10).unwrap());
    assert_eq!(inventory.find_by_id(&1).unwrap().unwrap().stock, 7);
    assert_eq!(
        publisher.accepted,
        vec![Placed {
            order_id: 10,
            quantity: 3
        }]
    );
}

#[test]
fn insufficient_stock_prevents_mutation_and_publication() {
    let mut orders = Store::new();
    let mut inventory = inventory(2);
    let mut publisher = Publisher::default();
    assert_eq!(
        place_order(&mut orders, &mut inventory, &mut publisher, 3),
        Err(PlaceError::InsufficientStock)
    );
    assert!(!orders.exists(&10).unwrap());
    assert_eq!(inventory.find_by_id(&1).unwrap().unwrap().stock, 2);
    assert!(publisher.accepted.is_empty());
}

#[test]
fn publication_failure_keeps_saves_and_returns_aggregate_for_retry() {
    let mut orders = Store::new();
    let mut inventory = inventory(10);
    let mut publisher = Publisher {
        fail: true,
        ..Publisher::default()
    };
    let error = place_order(&mut orders, &mut inventory, &mut publisher, 3).unwrap_err();
    let PlaceError::PublishFailed(mut order) = error else {
        panic!("expected publish failure")
    };
    assert!(orders.exists(&10).unwrap());
    assert_eq!(inventory.find_by_id(&1).unwrap().unwrap().stock, 7);
    assert_eq!(
        order.pull_events(),
        vec![Placed {
            order_id: 10,
            quantity: 3
        }]
    );
}
