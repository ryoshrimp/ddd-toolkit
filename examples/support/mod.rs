use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, Repository};
use std::{collections::HashMap, convert::Infallible};

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
pub struct Placed(pub u64);

#[derive(Debug, Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
pub struct Order {
    #[entity(id)]
    pub id: u64,
    pub total: u32,
    #[aggregate(events)]
    pub events: Vec<Placed>,
}

// Example-only storage: saving does not publish or drain the source queue.
#[derive(Default)]
pub struct Store {
    pub orders: HashMap<u64, Order>,
}
impl Repository for Store {
    type Aggregate = Order;
    type Error = Infallible;
    fn find_by_id(&self, id: &u64) -> Result<Option<Order>, Infallible> {
        Ok(self.orders.get(id).cloned())
    }
    fn save(&mut self, order: &Order) -> Result<(), Infallible> {
        let mut snapshot = order.clone();
        snapshot.pull_events();
        self.orders.insert(order.id, snapshot);
        Ok(())
    }
    fn delete(&mut self, id: &u64) -> Result<(), Infallible> {
        self.orders.remove(id);
        Ok(())
    }
    fn exists(&self, id: &u64) -> Result<bool, Infallible> {
        Ok(self.orders.contains_key(id))
    }
}

#[derive(Default)]
pub struct Publisher {
    pub accepted: Vec<Placed>,
    pub fail_at: Option<usize>,
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
