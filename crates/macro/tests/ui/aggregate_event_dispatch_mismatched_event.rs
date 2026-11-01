use ddd_toolkit_core::{dispatch_events, DomainEventPublisher};
use ddd_toolkit_macro::{AggregateRoot, DomainEvent, Entity};

#[derive(DomainEvent)]
struct AggregateEvent;

#[derive(DomainEvent)]
struct PublisherEvent;

#[derive(Entity, AggregateRoot)]
struct Aggregate {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<AggregateEvent>,
}

struct Publisher;

impl DomainEventPublisher for Publisher {
    type Event = PublisherEvent;
    type Error = ();

    fn publish(&mut self, _: &Self::Event) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    let mut aggregate = Aggregate {
        id: 1,
        events: Vec::new(),
    };
    let mut publisher = Publisher;
    let _ = dispatch_events(&mut aggregate, &mut publisher);
}
