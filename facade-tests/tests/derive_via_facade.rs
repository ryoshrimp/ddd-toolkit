// Derives are invoked through the facade without a direct core dependency.
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

#[derive(ddd_toolkit::ValueObject)]
struct NamedSingle {
    value: u32,
}

#[derive(ddd_toolkit::ValueObject)]
struct RawInner {
    r#inner: u32,
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = positive, error = &'static str)]
struct ValidatedSingle {
    value: u32,
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = positive, error = &'static str)]
struct NamedInner {
    inner: u32,
}

fn positive(value: &u32) -> Result<(), &'static str> {
    if *value > 0 { Ok(()) } else { Err("zero") }
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = ordered, error = &'static str)]
struct Range {
    start: u32,
    end: u32,
}

fn ordered(value: &Range) -> Result<(), &'static str> {
    if value.start <= value.end {
        Ok(())
    } else {
        Err("reversed")
    }
}

#[derive(ddd_toolkit::ValueObject)]
enum NamedVariant {
    One { count: u32 },
    Two { start: u32, end: u32 },
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = valid_variant, error = &'static str)]
enum ValidatedVariant {
    One { count: u32 },
    Two { start: u32, end: u32 },
}

fn valid_variant(value: &ValidatedVariant) -> Result<(), &'static str> {
    match value {
        ValidatedVariant::One { count } if *count > 0 => Ok(()),
        ValidatedVariant::Two { start, end } if start <= end => Ok(()),
        _ => Err("invalid"),
    }
}

#[test]
fn named_constructors_preserve_fields_and_validation() {
    assert_eq!(NamedSingle::new(4).into_inner(), 4);
    assert_eq!(RawInner::new(4).into_inner(), 4);
    assert_eq!(ValidatedSingle::try_new(4).unwrap().into_inner(), 4);
    assert_eq!(ValidatedSingle::try_new(0), Err("zero"));
    assert_eq!(NamedInner::try_new(4).unwrap().into_inner(), 4);
    assert_eq!(NamedInner::try_new(0), Err("zero"));
    let range = Range::try_new(2, 5).unwrap();
    assert_eq!((*range.start(), *range.end()), (2, 5));
    assert_eq!(Range::try_new(5, 2), Err("reversed"));
    assert_eq!(NamedVariant::new_one(3), NamedVariant::One { count: 3 });
    assert_eq!(
        NamedVariant::new_two(2, 5),
        NamedVariant::Two { start: 2, end: 5 }
    );
    assert_eq!(
        ValidatedVariant::try_new_one(3).unwrap(),
        ValidatedVariant::One { count: 3 }
    );
    assert_eq!(
        ValidatedVariant::try_new_two(2, 5).unwrap(),
        ValidatedVariant::Two { start: 2, end: 5 }
    );
    assert_eq!(ValidatedVariant::try_new_one(0), Err("invalid"));
    assert_eq!(ValidatedVariant::try_new_two(5, 2), Err("invalid"));
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
