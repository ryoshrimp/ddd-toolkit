use ddd_toolkit_core::AsyncDomainEventPublisher;
use std::future::Future;
use std::pin::Pin;

struct NotADomainEvent;
struct InvalidPublisher;

impl AsyncDomainEventPublisher for InvalidPublisher {
    type Event = NotADomainEvent;
    type Error = ();

    fn publish<'a>(
        &'a mut self,
        _: &'a Self::Event,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}

fn main() {}
