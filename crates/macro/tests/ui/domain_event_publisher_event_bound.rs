use ddd_toolkit_core::DomainEventPublisher;

struct NotADomainEvent;

struct InvalidPublisher;

impl DomainEventPublisher for InvalidPublisher {
    type Event = NotADomainEvent;
    type Error = ();

    fn publish(&mut self, _: &Self::Event) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {}
