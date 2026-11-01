//! CQRS: separate a command that adjusts an order total from a query that reads it.
//! CQRS: 注文金額を増やす命令と、金額を読む質問の責務を分ける。
//!
//! Run: cargo run -p ddd-toolkit --example cqrs --locked
//! Read main → AddToOrder::handle → SaveThenDispatch → Total, then support/mod.rs.
//! main → 命令の処理 → 保存/配信の順序 → クエリ → 共通アダプターの順に読む。
//! Expected: reject zero / totals above 5000, then query a total of 4200.
//! 0の加算と上限超過を拒否し、200を加算して4200を参照する。
//! Commands and queries share a repository; support/mod.rs defines the Order model.
//! 命令とクエリはRepositoryを共有する。Orderの定義はsupport/mod.rsを参照。
mod support;

use ddd_toolkit::{
    AggregateRoot as _, CommandHandler, LifecycleCallback, LifecycleCompletionError,
    LifecycleContext, LifecycleOperationError, Policy, QueryHandler, Repository, Specification,
};
use std::convert::Infallible;
use support::{Order, Placed, Publisher, Store};

// A reusable yes/no condition. Adapter failure and an unsatisfied rule differ.
// 再利用可能な真偽条件。条件不成立と評価自体の失敗は別。
struct Positive;
impl Specification<u32> for Positive {
    type Error = Infallible;
    fn is_satisfied_by(&self, amount: &u32) -> Result<bool, Infallible> {
        Ok(*amount > 0)
    }
}
// A constraint that can explain rejection; it does not persist anything.
// 拒否理由を返せる制約。Policy 自体は保存を行わない。
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
// Application orchestration, not a database transaction: dispatch can fail AFTER save.
// アプリケーションの手順。トランザクションではなく、保存成功後に配信だけ失敗し得る。
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
