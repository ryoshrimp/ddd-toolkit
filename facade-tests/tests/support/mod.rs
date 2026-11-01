use ddd_toolkit::{AggregateRoot, Repository};
use std::{collections::HashMap, convert::Infallible, hash::Hash};

// Test-local adapter: core 0.4 supplies contracts, not built-in storage.
// Saving a snapshot does not publish or drain the original aggregate's events.
pub(crate) struct Store<A: AggregateRoot> {
    values: HashMap<A::Id, A>,
}

impl<A: AggregateRoot> Store<A> {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl<A> Repository for Store<A>
where
    A: AggregateRoot + Clone,
    A::Id: Clone + Eq + Hash,
{
    type Aggregate = A;
    type Error = Infallible;

    fn find_by_id(&self, id: &A::Id) -> Result<Option<A>, Infallible> {
        Ok(self.values.get(id).cloned())
    }

    fn save(&mut self, aggregate: &A) -> Result<(), Infallible> {
        let mut snapshot = aggregate.clone();
        snapshot.pull_events();
        self.values.insert(aggregate.id().clone(), snapshot);
        Ok(())
    }

    fn delete(&mut self, id: &A::Id) -> Result<(), Infallible> {
        self.values.remove(id);
        Ok(())
    }

    fn exists(&self, id: &A::Id) -> Result<bool, Infallible> {
        Ok(self.values.contains_key(id))
    }
}
