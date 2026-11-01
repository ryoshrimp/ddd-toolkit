use ddd_toolkit_macro::{AggregateRoot, DomainEvent};

#[derive(DomainEvent)]
struct Event;

#[derive(AggregateRoot)]
struct NoEntity {
    #[aggregate(events)]
    events: Vec<Event>,
}

fn main() {}
