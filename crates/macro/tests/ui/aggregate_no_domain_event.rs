use ddd_toolkit_macro::{AggregateRoot, Entity};

struct Event;

#[derive(Entity, AggregateRoot)]
struct NoDomainEvent {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Event>,
}

fn main() {}
