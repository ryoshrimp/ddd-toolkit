// Macro 0.4.0 emits `field: field` in named ValueObject constructors.
// Clippy 1.99 checks those external macro spans; keep coverage until upstream
// generates shorthand fields. This allowance is limited to this test crate.
#![allow(clippy::redundant_field_names)]

// Derives are invoked through the facade; core is a direct dependency because
// macro 0.4 resolves its generated paths from the consumer's Cargo.toml.
use ddd_toolkit::AggregateRoot as _;

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = validate_email, error = &'static str)]
struct Email(String);

fn validate_email(value: &str) -> Result<(), &'static str> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("missing @")
    }
}

#[test]
fn value_object_validation_and_accessors() {
    fn assert_value_object<T: ddd_toolkit::ValueObject>() {}
    assert_value_object::<Email>();
    let email = Email::try_new("a@b.com".to_owned()).unwrap();
    assert_eq!(email.as_inner(), "a@b.com");
    assert_eq!(email.clone().into_inner(), "a@b.com");
    assert_eq!(Email::try_new("invalid".to_owned()), Err("missing @"));
}

#[derive(ddd_toolkit::ValueObject)]
struct Address {
    city: String,
    postal_code: u32,
}

#[derive(ddd_toolkit::ValueObject)]
struct Coordinates(i32, i32);

#[derive(ddd_toolkit::ValueObject)]
enum Status {
    Pending,
    Ready(u32),
}

#[test]
fn composite_value_objects_and_enum_accessors() {
    let address = Address::new("Tokyo".to_owned(), 100);
    assert_eq!(address.city(), "Tokyo");
    assert_eq!(address.postal_code(), &100);
    assert_eq!(ddd_toolkit::ValueObject::as_inner(&address), &address);
    let point = Coordinates::new(1, 2);
    assert_eq!(point.field_0(), &1);
    assert_eq!(point.field_1(), &2);
    let status = Status::new_ready(3);
    assert_eq!(status.as_ready(), Some((&3,)));
    assert!(!status.is_pending());
    assert!(Status::new_pending().is_pending());
}

#[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
enum Event {
    Placed,
    Cancelled,
}

#[derive(Debug, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Event>,
}

#[test]
fn entity_identity_and_aggregate_event_queue() {
    use std::hash::{DefaultHasher, Hash, Hasher};
    fn hash(value: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    let mut order = Order {
        id: 1,
        events: vec![],
    };
    let same_id = Order {
        id: 1,
        events: vec![Event::Cancelled],
    };
    let other_id = Order {
        id: 2,
        events: vec![],
    };
    assert_eq!(order.id(), &1);
    assert_eq!(ddd_toolkit::Entity::id(&order), &1);
    assert_eq!(order, same_id);
    assert_ne!(order, other_id);
    assert_eq!(hash(&order), hash(&same_id));
    order.record_event(Event::Placed);
    order.record_event(Event::Cancelled);
    assert_eq!(order.pull_events(), vec![Event::Placed, Event::Cancelled]);
    assert!(order.pull_events().is_empty());
}
