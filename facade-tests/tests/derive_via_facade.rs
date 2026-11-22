// Derives are invoked through the facade without a direct core dependency.
use ddd_toolkit::AggregateRoot as _;

extern crate alloc;

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
enum VariantInner {
    One { count: u32 },
    Two { start: u32, end: u32 },
}

#[derive(ddd_toolkit::ValueObject)]
#[value_object(validate = valid_variant, error = &'static str)]
struct ValidatedVariant(VariantInner);

fn valid_variant(value: &VariantInner) -> Result<(), &'static str> {
    match value {
        VariantInner::One { count } if *count > 0 => Ok(()),
        VariantInner::Two { start, end } if start <= end => Ok(()),
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
        ValidatedVariant::try_new(VariantInner::new_one(3))
            .unwrap()
            .into_inner(),
        VariantInner::One { count: 3 }
    );
    assert_eq!(
        ValidatedVariant::try_new(VariantInner::new_two(2, 5))
            .unwrap()
            .into_inner(),
        VariantInner::Two { start: 2, end: 5 }
    );
    assert_eq!(
        ValidatedVariant::try_new(VariantInner::new_one(0)),
        Err("invalid")
    );
    assert_eq!(
        ValidatedVariant::try_new(VariantInner::new_two(5, 2)),
        Err("invalid")
    );
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

#[derive(Debug, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
struct AllocOrder {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: alloc::vec::Vec<Event>,
}

#[test]
fn aggregate_event_queue_accepts_alloc_vec_path() {
    let mut order = AllocOrder {
        id: 1,
        events: vec![],
    };
    order.record_event(Event::Placed);
    assert_eq!(order.pull_events(), vec![Event::Placed]);
    assert!(order.pull_events().is_empty());
}

// Other derives must also resolve user-imported types in a nested consumer scope.
mod imported_types {
    use super::Email;
    use ddd_toolkit::AggregateRoot as _;
    use std::num::NonZeroU64;

    #[derive(Debug, Clone, PartialEq, ddd_toolkit::DomainEvent)]
    struct EmailChanged(Email);

    #[derive(Debug, ddd_toolkit::Entity, ddd_toolkit::AggregateRoot)]
    struct Account {
        #[entity(id)]
        id: NonZeroU64,
        #[aggregate(events)]
        events: Vec<EmailChanged>,
    }

    #[test]
    fn other_derives_preserve_imported_type_resolution() {
        let id = NonZeroU64::new(1).unwrap();
        let mut account = Account { id, events: vec![] };
        let event = EmailChanged(Email::try_new("a@b.com".to_owned()).unwrap());
        assert_eq!(ddd_toolkit::Entity::id(&account), &id);
        account.record_event(event.clone());
        assert_eq!(account.pull_events(), vec![event]);
    }
}

// no_eq / no_hash allow field types without Eq or Hash, alongside validate and serde.
mod without_eq_or_hash {
    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(no_eq, validate = finite, error = &'static str)]
    struct Price(f64);

    fn finite(value: &f64) -> Result<(), &'static str> {
        if value.is_finite() {
            Ok(())
        } else {
            Err("not finite")
        }
    }

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(no_eq)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(no_eq, serde)]
    struct Ratio(f64);

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct NoHash(u8);

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(no_hash)]
    struct Tag(NoHash);

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(no_eq)]
    enum Measure {
        Unset,
        Value(f64),
    }

    #[test]
    fn serde_round_trips_with_no_eq() {
        let json = serde_json::to_string(&Ratio::new(0.5)).unwrap();
        assert_eq!(json, "0.5");
        assert!(serde_json::from_str::<Ratio>(&json).unwrap() == Ratio::new(0.5));
    }

    #[test]
    fn partial_eq_clone_and_debug_remain() {
        let price = Price::try_new(1.5).unwrap();
        assert_eq!(price.clone(), price);
        assert_eq!(format!("{price:?}"), "Price(1.5)");
        assert_eq!(Price::try_new(f64::NAN), Err("not finite"));

        let point = Point::new(1.0, 2.0);
        assert!(point.clone() == point);

        let tag = Tag::new(NoHash(1));
        assert!(tag.clone() == tag);

        let measure = Measure::new_value(1.0);
        assert!(measure.clone() == measure);
        assert!(measure != Measure::new_unset());
    }
}

// Single-field Debug follows the definition shape and includes the type name.
mod single_field_debug {
    #[derive(ddd_toolkit::ValueObject)]
    struct Named {
        value: String,
    }

    #[derive(ddd_toolkit::ValueObject)]
    struct Tuple(String);

    #[derive(ddd_toolkit::ValueObject)]
    struct Generic<T> {
        value: T,
    }

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(validate = non_empty, error = &'static str)]
    struct Validated(String);

    fn non_empty(value: &str) -> Result<(), &'static str> {
        if value.is_empty() {
            Err("empty")
        } else {
            Ok(())
        }
    }

    #[derive(ddd_toolkit::ValueObject)]
    struct Raw {
        r#type: u32,
    }

    #[derive(ddd_toolkit::ValueObject)]
    #[allow(non_camel_case_types)]
    struct r#match(u32);

    #[test]
    fn includes_type_name_and_follows_shape() {
        let named = Named::new("a".to_owned());
        assert_eq!(format!("{named:?}"), r#"Named { value: "a" }"#);
        assert_eq!(format!("{named:#?}"), "Named {\n    value: \"a\",\n}");

        let tuple = Tuple::new("a".to_owned());
        assert_eq!(format!("{tuple:?}"), r#"Tuple("a")"#);
        assert_eq!(format!("{tuple:#?}"), "Tuple(\n    \"a\",\n)");

        assert_eq!(format!("{:?}", Generic::new(1u8)), "Generic { value: 1 }");
        assert_eq!(
            format!("{:?}", Validated::try_new("a".to_owned()).unwrap()),
            r#"Validated("a")"#
        );
        assert_eq!(format!("{:?}", Raw::new(1)), "Raw { type: 1 }");
        assert_eq!(format!("{:?}", r#match::new(1)), "match(1)");
    }
}
