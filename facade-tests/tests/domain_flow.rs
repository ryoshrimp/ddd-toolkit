mod support;

use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, Repository, dispatch_events};
use support::Store;

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct Placed(u64);

#[derive(Debug, Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    total: u32,
    #[aggregate(events)]
    events: Vec<Placed>,
}

#[test]
fn repository_save_and_publication_are_separate() {
    let mut store = Store::new();
    let mut order = Order {
        id: 1,
        total: 4200,
        events: vec![],
    };
    order.record_event(Placed(1));
    store.save(&order).unwrap();
    assert!(store.exists(&1).unwrap());
    let mut loaded = store.find_by_id(&1).unwrap().unwrap();
    assert_eq!(loaded.total, 4200);
    assert!(loaded.pull_events().is_empty());
    assert_eq!(order.pull_events(), vec![Placed(1)]);
    store.delete(&1).unwrap();
    assert!(store.find_by_id(&1).unwrap().is_none());
}

struct Publisher {
    accepted: Vec<Placed>,
    fail_at: Option<usize>,
}
impl DomainEventPublisher for Publisher {
    type Event = Placed;
    type Error = &'static str;
    fn publish(&mut self, event: &Placed) -> Result<(), Self::Error> {
        if self.fail_at == Some(self.accepted.len()) {
            return Err("broker down");
        }
        self.accepted.push(event.clone());
        Ok(())
    }
}

#[test]
fn dispatch_failure_restores_all_events_and_retry_may_duplicate() {
    let mut order = Order {
        id: 1,
        total: 4200,
        events: vec![Placed(1), Placed(2)],
    };
    let mut publisher = Publisher {
        accepted: vec![],
        fail_at: Some(1),
    };
    assert_eq!(
        dispatch_events(&mut order, &mut publisher),
        Err("broker down")
    );
    assert_eq!(order.events, vec![Placed(1), Placed(2)]);
    assert_eq!(publisher.accepted, vec![Placed(1)]);
    publisher.fail_at = None;
    dispatch_events(&mut order, &mut publisher).unwrap();
    assert!(order.pull_events().is_empty());
    assert_eq!(publisher.accepted, vec![Placed(1), Placed(1), Placed(2)]);
}
