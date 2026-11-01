use ddd_toolkit_core::{AsyncRepository, Entity};
use std::future::Future;
use std::pin::Pin;

struct NotAggregate {
    id: u64,
}

impl Entity for NotAggregate {
    type Id = u64;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

struct InvalidRepository;

impl AsyncRepository for InvalidRepository {
    type Aggregate = NotAggregate;
    type Error = ();

    fn find_by_id<'a>(
        &'a self,
        _: &'a u64,
    ) -> Pin<Box<dyn Future<Output = Result<Option<NotAggregate>, ()>> + Send + 'a>> {
        Box::pin(async { Ok(None) })
    }

    fn save<'a>(
        &'a mut self,
        _: &'a NotAggregate,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }

    fn delete<'a>(
        &'a mut self,
        _: &'a u64,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }

    fn exists<'a>(
        &'a self,
        _: &'a u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool, ()>> + Send + 'a>> {
        Box::pin(async { Ok(false) })
    }
}

fn main() {}
