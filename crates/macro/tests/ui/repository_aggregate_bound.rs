use ddd_toolkit_core::{Entity, Repository};

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

impl Repository for InvalidRepository {
    type Aggregate = NotAggregate;
    type Error = ();

    fn find_by_id(&self, _: &u64) -> Result<Option<NotAggregate>, ()> {
        Ok(None)
    }

    fn save(&mut self, _: &NotAggregate) -> Result<(), ()> {
        Ok(())
    }

    fn delete(&mut self, _: &u64) -> Result<(), ()> {
        Ok(())
    }

    fn exists(&self, _: &u64) -> Result<bool, ()> {
        Ok(false)
    }
}

fn main() {}
