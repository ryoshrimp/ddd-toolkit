use ddd_toolkit_macro::{AggregateRoot, Entity};

mod fake {
    /// **English:** Mimics a vector name without providing a standard event queue.
    ///
    /// **日本語:** 標準イベントキューではない型にベクターの名前を付けます。
    pub struct Vec<T>(core::marker::PhantomData<T>);
}

#[derive(Entity, AggregateRoot)]
struct FakeQueue {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: fake::Vec<u64>,
}

fn main() {}
