use ddd_toolkit_core::{dispatch_events_async, AggregateRoot, AsyncDomainEventPublisher, DomainEvent, Entity};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;

struct NonSyncEvent(Cell<u64>);
impl DomainEvent for NonSyncEvent {}

struct Aggregate {
    id: u64,
    events: Vec<NonSyncEvent>,
}

impl Entity for Aggregate {
    type Id = u64;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl AggregateRoot for Aggregate {
    type Event = NonSyncEvent;

    fn record_event(&mut self, event: Self::Event) {
        self.events.push(event);
    }

    fn pull_events(&mut self) -> Vec<Self::Event> {
        std::mem::take(&mut self.events)
    }
}

struct Publisher;

impl AsyncDomainEventPublisher for Publisher {
    type Event = NonSyncEvent;
    type Error = ();

    fn publish<'a>(
        &'a mut self,
        _: &'a Self::Event,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}

fn main() {
    let mut aggregate = Aggregate {
        id: 1,
        events: vec![NonSyncEvent(Cell::new(1))],
    };
    let mut publisher = Publisher;
    let _ = dispatch_events_async(&mut aggregate, &mut publisher);
}
