//! Advanced delivery exercise, NOT the first tutorial: start with getting_started.rs.
//! 発展編（最初は getting_started.rs）: 保存後の配信が部分失敗するとどうなるか。
//!
//! Run: cargo run -p ddd-toolkit --example basic --locked
//! Read main first, then support/mod.rs for the in-memory adapters.
//! main → support/mod.rs の順に読む。これは業務モデルより配信契約に絞った例。
//! Expected: the restored batch retries event 1, producing deliveries [1, 1, 2].
//! 結果: 先頭の配信成功は取り消せないため、バッチ再試行で [1, 1, 2] となる。
mod support;

use ddd_toolkit::{AggregateRoot as _, Repository, dispatch_events};
use support::{Order, Placed, Publisher, Store};

fn main() {
    let mut order = Order {
        id: 1,
        total: 4200,
        events: vec![],
    };
    order.record_event(Placed(1));
    order.record_event(Placed(2));
    let mut store = Store::default();
    store.save(&order).unwrap();
    let mut loaded = store.find_by_id(&1).unwrap().unwrap();
    assert_eq!(loaded.total, 4200);
    assert!(loaded.pull_events().is_empty());

    // Saving succeeded. A later publisher error cannot undo that save.
    // 保存と配信は別操作。後の配信失敗で保存がロールバックされるわけではない。
    let mut publisher = Publisher {
        fail_at: Some(1),
        ..Publisher::default()
    };
    assert_eq!(
        dispatch_events(&mut order, &mut publisher),
        Err("broker down")
    );
    assert!(store.exists(&1).unwrap()); // Publication failure does not roll back saving.
    assert_eq!(order.events, vec![Placed(1), Placed(2)]);
    // The whole batch was restored, including the event already delivered.
    // 配信済みイベントも含めたバッチ全体が復元される。受信側には重複対策が必要。
    publisher.fail_at = None;
    dispatch_events(&mut order, &mut publisher).unwrap();
    assert!(order.pull_events().is_empty());
    assert_eq!(publisher.accepted, vec![Placed(1), Placed(1), Placed(2)]);
    println!("basic: saved total=4200; retry delivered [1, 1, 2]");
}
