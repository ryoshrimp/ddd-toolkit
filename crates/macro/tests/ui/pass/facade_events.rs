use ddd_toolkit_core::AggregateRoot;
use ddd_toolkit_macro::{AggregateRoot, DomainEvent, Entity};

#[derive(DomainEvent)]
enum Event {
    Created,
}

#[derive(Entity, AggregateRoot)]
struct Root {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Event>,
}

fn main() {
    let mut root = Root {
        id: 1,
        events: Vec::new(),
    };
    root.record_event(Event::Created);
    assert_eq!(root.pull_events().len(), 1);
}
