//! Start here: build a draft order, confirm it, save it, then notify a subscriber.
//! 最初に読む例: 下書きの注文に商品を追加し、確定・保存してから通知する。
//! Read alongside README.md / README.ja.md in this directory.
use ddd_toolkit::{AggregateRoot as _, DomainEventPublisher, Repository, dispatch_events};
use std::{collections::HashMap, convert::Infallible};

// 1. DOMAIN / 業務モデル
// A fact in the past tense, not a request to perform an action.
// 過去に起きた事実。「確定せよ」という命令ではなく「確定された」という通知。
#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
struct OrderConfirmed {
    order_id: u64,
    total_yen: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderError {
    ZeroPrice,
    TotalOverflow,
    EmptyOrder,
    AlreadyConfirmed,
}

// Aggregate methods enforce business rules; derives implement toolkit traits.
// Private fields restrict direct access to this module; main also uses the methods.
// 集約のメソッドで業務ルールを実装し、deriveでtoolkitのtraitを実装する。
// 非公開フィールドに直接アクセスできるのは同じモジュール内。mainも変更はメソッド経由。
#[derive(Clone, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    total_yen: u32,
    confirmed: bool,
    #[aggregate(events)]
    events: Vec<OrderConfirmed>,
}
impl Order {
    fn draft(id: u64) -> Self {
        Self {
            id,
            total_yen: 0,
            confirmed: false,
            events: vec![],
        }
    }

    // In this deliberately small shop, add one item at a time in whole yen.
    // 教材用に、商品は1個ずつ・価格は整数の円で扱う（税・配送・決済は対象外）。
    fn add_item(&mut self, price_yen: u32) -> Result<(), OrderError> {
        if self.confirmed {
            return Err(OrderError::AlreadyConfirmed);
        }
        if price_yen == 0 {
            return Err(OrderError::ZeroPrice);
        }
        self.total_yen = self
            .total_yen
            .checked_add(price_yen)
            .ok_or(OrderError::TotalOverflow)?;
        Ok(())
    }

    fn confirm(&mut self) -> Result<(), OrderError> {
        if self.confirmed {
            return Err(OrderError::AlreadyConfirmed);
        }
        if self.total_yen == 0 {
            return Err(OrderError::EmptyOrder);
        }
        self.confirmed = true;
        // Recording is local; it does NOT send anything to a broker.
        // 記録は保留キューへの追加だけ。この時点では外部に通知しない。
        self.record_event(OrderConfirmed {
            order_id: self.id,
            total_yen: self.total_yen,
        });
        Ok(())
    }
}

// 2. APPLICATION / アプリケーションの手順
// Read this before the adapter implementations below.
// 下のアダプター実装より先に、この関数を読む。
fn main() {
    let mut order = Order::draft(1);
    assert_eq!(order.confirm(), Err(OrderError::EmptyOrder));
    assert_eq!(order.add_item(0), Err(OrderError::ZeroPrice));
    order.add_item(1200).unwrap();
    order.add_item(3000).unwrap();
    order.confirm().unwrap();
    assert_eq!(order.add_item(100), Err(OrderError::AlreadyConfirmed));
    assert_eq!(order.confirm(), Err(OrderError::AlreadyConfirmed));
    assert_eq!(order.total_yen, 4200);
    assert_eq!(order.events.len(), 1); // Rejected operations did not emit more events.

    // Persistence and notification are separate, explicitly ordered operations.
    // 保存と通知は別操作。順番はアプリケーションが明示する。
    let mut orders = MemoryOrders::default();
    orders.save(&order).unwrap();
    let saved = orders.find_by_id(&1).unwrap().unwrap();
    assert!(saved.confirmed);
    assert_eq!(saved.total_yen, 4200);
    assert!(saved.events.is_empty()); // Persisted order state excludes pending notifications.
    assert_eq!(order.events.len(), 1); // Saving did not consume the original queue.

    let mut notifications = Notifications::default();
    dispatch_events(&mut order, &mut notifications).unwrap();
    assert_eq!(
        notifications.sent,
        vec![OrderConfirmed {
            order_id: 1,
            total_yen: 4200
        }]
    );
    assert!(order.events.is_empty());
    println!("getting_started: order #1 confirmed; saved total=4200 yen; notifications=1");
}

// 3. ADAPTERS / 外部との接続部分（読み飛ばしてから戻ってよい）
// A repository speaks in aggregates, not SQL rows. This adapter stores a clone,
// without pending notifications. It is not durable and has no concurrency control.
// Repository は集約単位で保存・取得する契約。ここでは複製をメモリに保存するだけ。
// 保留通知は保存しない。永続性・同時更新制御はない。
#[derive(Default)]
struct MemoryOrders(HashMap<u64, Order>);
impl Repository for MemoryOrders {
    type Aggregate = Order;
    type Error = Infallible;
    fn find_by_id(&self, id: &u64) -> Result<Option<Order>, Infallible> {
        Ok(self.0.get(id).cloned())
    }
    fn save(&mut self, order: &Order) -> Result<(), Infallible> {
        let mut saved = order.clone();
        saved.pull_events();
        self.0.insert(order.id, saved);
        Ok(())
    }
    fn delete(&mut self, id: &u64) -> Result<(), Infallible> {
        self.0.remove(id);
        Ok(())
    }
    fn exists(&self, id: &u64) -> Result<bool, Infallible> {
        Ok(self.0.contains_key(id))
    }
}

// A stand-in for a subscriber that would arrange a confirmation email.
// 確認メールを手配する購読者の代役。実際のメール送信は行わない。
#[derive(Default)]
struct Notifications {
    sent: Vec<OrderConfirmed>,
}
impl DomainEventPublisher for Notifications {
    type Event = OrderConfirmed;
    type Error = Infallible;
    fn publish(&mut self, event: &OrderConfirmed) -> Result<(), Infallible> {
        self.sent.push(event.clone());
        Ok(())
    }
}
