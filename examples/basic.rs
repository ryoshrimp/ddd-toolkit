//! Save an aggregate, then publish; a partial failure can duplicate delivery.
mod support;

use ddd_toolkit::{AggregateRoot as _, Repository, dispatch_events};
use support::{Order, Placed, Publisher, Store};

fn main() {
    let mut order = Order {
        id: 1,
        total: 4200,
        events: vec![],
    };
    order.record_event(Placed(1));
    order.record_event(Placed(2));
    let mut store = Store::default();
    store.save(&order).unwrap();
    let mut loaded = store.find_by_id(&1).unwrap().unwrap();
    assert_eq!(loaded.total, 4200);
    assert!(loaded.pull_events().is_empty());

    let mut publisher = Publisher {
        fail_at: Some(1),
        ..Publisher::default()
    };
    assert_eq!(
        dispatch_events(&mut order, &mut publisher),
        Err("broker down")
    );
    assert!(store.exists(&1).unwrap()); // Publication failure does not roll back saving.
    assert_eq!(order.events, vec![Placed(1), Placed(2)]);
    publisher.fail_at = None;
    dispatch_events(&mut order, &mut publisher).unwrap();
    assert!(order.pull_events().is_empty());
    assert_eq!(publisher.accepted, vec![Placed(1), Placed(1), Placed(2)]);
    println!("basic: saved total=4200; retry delivered [1, 1, 2]");
}
