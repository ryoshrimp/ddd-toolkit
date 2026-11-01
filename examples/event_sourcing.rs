//! Zero-based revisions, schema upcasting, replay, and optional snapshots.
use ddd_toolkit::{
    AggregateReplay, AggregateRoot as _, AggregateSnapshot, AppendError, AppendResult,
    CurrentRevision, EventStore, EventUpcaster, ExpectedRevision, LoadResult, NoStream,
    RevisionConflict, SnapshotLoadError, SnapshotSaveError, SnapshotStore, SnapshotUse,
    StreamHistory, StreamRevision, replay_aggregate, replay_with_optional_snapshot,
};
use std::{collections::HashMap, convert::Infallible};

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
enum Added {
    V1(u32),
    V2 { amount: u32 },
}
struct ToV2;
impl EventUpcaster<Added> for ToV2 {
    type Error = Infallible;
    fn upcast(&self, event: Added) -> Result<Added, Infallible> {
        Ok(match event {
            Added::V1(amount) => Added::V2 { amount },
            current => current,
        })
    }
}
#[derive(Debug, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Counter {
    #[entity(id)]
    id: u64,
    value: u32,
    #[aggregate(events)]
    events: Vec<Added>,
}
impl AggregateReplay for Counter {
    type InitializationError = Infallible;
    type ApplicationError = &'static str;
    type Snapshot = u32;
    type SnapshotValidationError = Infallible;
    type SnapshotStateError = Infallible;
    fn initial_state(id: &u64) -> Result<Self, Infallible> {
        Ok(Self {
            id: *id,
            value: 0,
            events: vec![],
        })
    }
    fn apply_event(&mut self, event: &Added) -> Result<(), Self::ApplicationError> {
        let Added::V2 { amount } = event else {
            return Err("upcast before replay");
        };
        self.value = self.value.checked_add(*amount).ok_or("counter overflow")?;
        Ok(()) // Historical events must not become pending events.
    }
    fn classify_snapshot(
        _: &AggregateSnapshot<u32>,
        _: StreamRevision,
    ) -> Result<SnapshotUse, Infallible> {
        // This example has only one snapshot schema.
        Ok(SnapshotUse::Applicable)
    }
    fn from_snapshot(id: &u64, snapshot: AggregateSnapshot<u32>) -> Result<Self, Infallible> {
        Ok(Self {
            id: *id,
            value: snapshot.into_parts().0,
            events: vec![],
        })
    }
}

#[derive(Default)]
struct MemoryEvents(HashMap<u64, Vec<Added>>);
impl EventStore for MemoryEvents {
    type Aggregate = Counter;
    type Error = Infallible;
    fn load(&self, id: &u64) -> Result<LoadResult<Added>, Infallible> {
        match self.0.get(id) {
            None => Ok(LoadResult::NoStream(NoStream)),
            Some(events) => {
                // Real adapters deserialize and select migrations at this boundary.
                let current = events
                    .iter()
                    .cloned()
                    .map(|event| ToV2.upcast(event).unwrap())
                    .collect();
                Ok(LoadResult::Found(
                    StreamHistory::try_new(current, StreamRevision::new(events.len() as u64 - 1))
                        .unwrap(),
                ))
            }
        }
    }
    fn append(
        &mut self,
        id: &u64,
        expected: ExpectedRevision,
        events: Vec<Added>,
    ) -> Result<AppendResult, AppendError<Infallible>> {
        if events.is_empty() {
            return Err(AppendError::EmptyBatch);
        }
        let current = self
            .0
            .get(id)
            .map(|events| StreamRevision::new(events.len() as u64 - 1));
        let matches = match (expected, current) {
            (ExpectedRevision::NoStream(_), None) => true,
            (ExpectedRevision::Revision(want), Some(have)) => want == have,
            _ => false,
        };
        if !matches {
            return Err(AppendError::Conflict(RevisionConflict {
                expected,
                current: current.map_or(
                    CurrentRevision::NoStream(NoStream),
                    CurrentRevision::Revision,
                ),
            }));
        }
        let revision = match current {
            None => StreamRevision::new(events.len() as u64 - 1),
            Some(revision) => StreamRevision::new(
                revision
                    .value()
                    .checked_add(events.len() as u64)
                    .ok_or(AppendError::RevisionExhausted { current: revision })?,
            ),
        };
        // Exclusive access makes the check + append atomic only in this process.
        self.0.entry(*id).or_default().extend(events);
        Ok(AppendResult {
            resulting_revision: revision,
        })
    }
}
#[derive(Default)]
struct MemorySnapshots(HashMap<u64, AggregateSnapshot<u32>>);
impl SnapshotStore for MemorySnapshots {
    type Aggregate = Counter;
    type State = u32;
    type Error = Infallible;
    fn load(
        &self,
        id: &u64,
    ) -> Result<Option<AggregateSnapshot<u32>>, SnapshotLoadError<Infallible>> {
        Ok(self
            .0
            .get(id)
            .map(|snapshot| AggregateSnapshot::new(*snapshot.state(), snapshot.revision())))
    }
    fn save(
        &mut self,
        id: &u64,
        snapshot: AggregateSnapshot<u32>,
    ) -> Result<(), SnapshotSaveError<Infallible>> {
        self.0.insert(*id, snapshot);
        Ok(())
    }
}
fn history(store: &MemoryEvents) -> StreamHistory<Added> {
    let LoadResult::Found(history) = store.load(&1).unwrap() else {
        panic!("stream missing");
    };
    history
}
fn main() {
    let mut store = MemoryEvents::default();
    assert!(matches!(store.load(&1).unwrap(), LoadResult::NoStream(_)));
    let first = store
        .append(&1, ExpectedRevision::NoStream(NoStream), vec![Added::V1(2)])
        .unwrap();
    assert_eq!(first.resulting_revision, StreamRevision::new(0));
    let mut replayed = replay_aggregate::<Counter>(&1, &history(&store)).unwrap();
    assert_eq!(replayed.aggregate.value, 2);
    assert!(replayed.aggregate.pull_events().is_empty());
    let mut snapshots = MemorySnapshots::default();
    snapshots
        .save(
            &1,
            AggregateSnapshot::new(replayed.aggregate.value, replayed.revision),
        )
        .unwrap();
    store
        .append(
            &1,
            ExpectedRevision::Revision(replayed.revision),
            vec![Added::V2 { amount: 3 }],
        )
        .unwrap();
    assert!(matches!(
        store.append(
            &1,
            ExpectedRevision::Revision(replayed.revision),
            vec![Added::V1(99)]
        ),
        Err(AppendError::Conflict(_))
    ));
    let history = history(&store);
    let mut restored =
        replay_with_optional_snapshot::<Counter>(&1, &history, snapshots.load(&1).unwrap())
            .unwrap();
    let full = replay_aggregate::<Counter>(&1, &history).unwrap();
    assert_eq!(restored.aggregate.value, 5);
    assert_eq!(restored.aggregate.value, full.aggregate.value);
    assert_eq!(restored.revision, StreamRevision::new(1));
    assert!(restored.aggregate.pull_events().is_empty());
    println!("event_sourcing: upcast v1; rejected stale write; snapshot + tail = 5");
}
