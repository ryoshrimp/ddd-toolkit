//! A command validates rules and invokes an explicit save/dispatch lifecycle.
mod support;

use ddd_toolkit::{
    AggregateRoot as _, CommandHandler, LifecycleCallback, LifecycleCompletionError,
    LifecycleContext, LifecycleOperationError, Policy, QueryHandler, Repository, Specification,
};
use std::convert::Infallible;
use support::{Order, Placed, Publisher, Store};

struct Positive;
impl Specification<u32> for Positive {
    type Error = Infallible;
    fn is_satisfied_by(&self, amount: &u32) -> Result<bool, Infallible> {
        Ok(*amount > 0)
    }
}
struct Limit;
impl Policy<u32> for Limit {
    type Error = Error;
    fn check(&self, total: &u32) -> Result<(), Error> {
        if *total <= 5000 {
            Ok(())
        } else {
            Err(Error::Rejected)
        }
    }
}

#[derive(Debug, PartialEq)]
enum Error {
    NotFound,
    Rejected,
    Operation(LifecycleOperationError<&'static str>),
    Completion(LifecycleCompletionError),
}
impl From<LifecycleOperationError<Infallible>> for Error {
    fn from(error: LifecycleOperationError<Infallible>) -> Self {
        match error {
            LifecycleOperationError::Adapter(never) => match never {},
            LifecycleOperationError::AlreadySaved => {
                Self::Operation(LifecycleOperationError::AlreadySaved)
            }
            LifecycleOperationError::AlreadyDispatched => {
                Self::Operation(LifecycleOperationError::AlreadyDispatched)
            }
        }
    }
}
impl From<LifecycleOperationError<&'static str>> for Error {
    fn from(error: LifecycleOperationError<&'static str>) -> Self {
        Self::Operation(error)
    }
}
impl From<LifecycleCompletionError> for Error {
    fn from(error: LifecycleCompletionError) -> Self {
        Self::Completion(error)
    }
}

struct AddToOrder {
    store: Store,
    publisher: Publisher,
}
impl CommandHandler for AddToOrder {
    type Command = u32;
    type Aggregate = Order;
    type Repository = Store;
    type Publisher = Publisher;
    type Output = ();
    type Error = Error;
    fn handle<F>(&mut self, id: &u64, amount: u32, callback: F) -> Result<(), Error>
    where
        F: for<'a> LifecycleCallback<'a, Order, Store, Publisher, Output = (), Error = Error>,
    {
        let mut order = self.store.find_by_id(id).unwrap().ok_or(Error::NotFound)?;
        if !Positive.is_satisfied_by(&amount).unwrap() {
            return Err(Error::Rejected);
        }
        let total = order.total.checked_add(amount).ok_or(Error::Rejected)?;
        Limit.check(&total)?;
        order.total = total;
        order.record_event(Placed(order.id));
        let mut context = LifecycleContext::new(&mut order, &mut self.store, &mut self.publisher);
        callback.call(&mut context)?;
        context.complete()?;
        Ok(())
    }
}
struct SaveThenDispatch;
impl<'storage> LifecycleCallback<'storage, Order, Store, Publisher> for SaveThenDispatch {
    type Output = ();
    type Error = Error;
    fn call<'callback>(
        self,
        context: &'callback mut LifecycleContext<'storage, Order, Store, Publisher>,
    ) -> Result<(), Error>
    where
        'storage: 'callback,
    {
        context.save()?;
        context.dispatch()?;
        Ok(())
    }
}
struct Total<'a>(&'a Store);
impl QueryHandler for Total<'_> {
    type Query = u64;
    type Result = u32;
    type Error = Error;
    fn handle(&self, id: u64) -> Result<u32, Error> {
        self.0
            .find_by_id(&id)
            .unwrap()
            .map(|order| order.total)
            .ok_or(Error::NotFound)
    }
}

fn main() {
    let mut handler = AddToOrder {
        store: Store::default(),
        publisher: Publisher::default(),
    };
    handler
        .store
        .save(&Order {
            id: 1,
            total: 4000,
            events: vec![],
        })
        .unwrap();
    assert_eq!(
        handler.handle(&1, 0, SaveThenDispatch),
        Err(Error::Rejected)
    );
    assert_eq!(
        handler.handle(&1, 1001, SaveThenDispatch),
        Err(Error::Rejected)
    );
    assert_eq!(Total(&handler.store).handle(1), Ok(4000));
    assert!(handler.publisher.accepted.is_empty());
    handler.handle(&1, 200, SaveThenDispatch).unwrap();
    assert_eq!(Total(&handler.store).handle(1), Ok(4200));
    assert_eq!(handler.publisher.accepted, vec![Placed(1)]);
    println!("cqrs: rejected invalid commands; query total=4200");
}
