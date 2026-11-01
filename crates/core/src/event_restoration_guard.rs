use crate::AggregateRoot;

pub(crate) struct EventRestorationGuard<'a, A: AggregateRoot> {
    aggregate: &'a mut A,
    snapshot: Option<Vec<A::Event>>,
}

impl<'a, A: AggregateRoot> EventRestorationGuard<'a, A> {
    pub(crate) fn new(aggregate: &'a mut A, snapshot: Vec<A::Event>) -> Self {
        Self {
            aggregate,
            snapshot: Some(snapshot),
        }
    }

    pub(crate) fn events(&self) -> impl Iterator<Item = &A::Event> {
        self.snapshot.as_deref().into_iter().flatten()
    }

    pub(crate) fn commit(&mut self) {
        self.snapshot.take();
    }
}

impl<A: AggregateRoot> Drop for EventRestorationGuard<'_, A> {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            for event in snapshot {
                self.aggregate.record_event(event);
            }
        }
    }
}
