//! `adapter`/`mock` clock and id-generator implementations, reachable and
//! working purely via `ddd_toolkit::` paths.

use ddd_toolkit::domain::{EntityId, ValueObject};
use ddd_toolkit::{
    adapter::{clock::SystemClock, id::UuidV4Generator},
    mock::clock::FixedClock,
    port::{clock::Clock, id::IdGenerator},
};
use std::fmt::Display;

#[test]
fn system_clock_returns_current_time_through_facade() {
    let before = chrono::Utc::now();
    let now = SystemClock.now();
    assert!((now - before).num_seconds().abs() < 5);
}

#[test]
fn fixed_clock_returns_configured_time_through_facade() {
    let time = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let clock = FixedClock::new(time);
    assert_eq!(clock.now(), time);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UuidUserId(uuid::Uuid);

impl ValueObject for UuidUserId {}

impl Display for UuidUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl EntityId for UuidUserId {}

impl From<uuid::Uuid> for UuidUserId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

#[test]
fn uuid_v4_generator_produces_distinct_ids_through_facade() {
    let generator = UuidV4Generator::<UuidUserId>::new();

    let a = generator.generate();
    let b = generator.generate();

    assert_ne!(a, b);
}
