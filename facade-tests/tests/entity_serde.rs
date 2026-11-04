use std::sync::atomic::{AtomicUsize, Ordering};

static VALIDATIONS: AtomicUsize = AtomicUsize::new(0);

// Neither Display nor Debug is required for validation errors.
struct InvalidRange;

#[derive(Debug, ddd_toolkit::Entity)]
#[entity(validate = validate_order, error = InvalidRange, serde)]
struct Order {
    #[entity(id)]
    id: u64,
    minimum: u32,
    maximum: u32,
}

fn validate_order(order: &Order) -> Result<(), InvalidRange> {
    VALIDATIONS.fetch_add(1, Ordering::SeqCst);
    if order.minimum <= order.maximum {
        Ok(())
    } else {
        Err(InvalidRange)
    }
}

#[test]
fn construction_and_restoration_validate_the_supplied_state_once() {
    VALIDATIONS.store(0, Ordering::SeqCst);
    assert!(Order::try_new(42, 5, 2).is_err());
    assert_eq!(VALIDATIONS.load(Ordering::SeqCst), 1);
    let order = Order::try_new(42, 2, 5).ok().unwrap();
    assert_eq!(VALIDATIONS.load(Ordering::SeqCst), 2);
    let json = serde_json::to_string(&order).unwrap();
    assert_eq!(json, r#"{"id":42,"minimum":2,"maximum":5}"#);
    let restored: Order = serde_json::from_str(&json).unwrap();
    assert_eq!(VALIDATIONS.load(Ordering::SeqCst), 3);
    assert_eq!(restored.id(), &42);
    assert_eq!((restored.minimum, restored.maximum), (2, 5));
    assert_eq!(order, restored);
    let error = serde_json::from_str::<Order>(r#"{"id":42,"minimum":5,"maximum":2}"#).unwrap_err();
    assert!(error.to_string().contains("entity validation failed"));
    assert_eq!(VALIDATIONS.load(Ordering::SeqCst), 4);
    assert!(serde_json::from_str::<Order>(r#"{"id":42,"minimum":2}"#).is_err());
    assert_eq!(VALIDATIONS.load(Ordering::SeqCst), 4);
    // State differs, identity semantics do not.
    assert_eq!(
        order,
        Order {
            id: 42,
            minimum: 0,
            maximum: 10
        }
    );
}

#[derive(Debug, ddd_toolkit::Entity)]
#[entity(serde)]
struct GenericEntity<T, const N: usize>
where
    T: Clone,
{
    #[entity(id)]
    id: T,
    r#type: Vec<u8>,
}

#[derive(Debug, ddd_toolkit::Entity)]
#[entity(serde)]
struct IdentityOnly {
    #[entity(id)]
    id: u64,
}

#[test]
fn serde_only_supports_generics_and_single_field_struct_shape() {
    let value = GenericEntity::<String, 3> {
        id: "abc".into(),
        r#type: vec![1, 2],
    };
    let json = serde_json::to_string(&value).unwrap();
    let restored: GenericEntity<String, 3> = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, value);
    assert_eq!(restored.r#type, value.r#type);
    assert_eq!(json, r#"{"id":"abc","type":[1,2]}"#);
    let json = serde_json::to_string(&IdentityOnly { id: 7 }).unwrap();
    assert_eq!(json, r#"{"id":7}"#);
    assert_eq!(
        serde_json::from_str::<IdentityOnly>(&json).unwrap().id(),
        &7
    );
}

#[derive(ddd_toolkit::Entity)]
#[entity(validate = validate_identity, error = &'static str)]
struct ValidatedIdentity {
    #[entity(id)]
    id: u64,
}

fn validate_identity(value: &ValidatedIdentity) -> Result<(), &'static str> {
    if value.id == 0 {
        Err("zero identity")
    } else {
        Ok(())
    }
}

#[test]
fn validation_without_serde_receives_self_even_for_one_field() {
    assert!(matches!(
        ValidatedIdentity::try_new(0),
        Err("zero identity")
    ));
    assert_eq!(ValidatedIdentity::try_new(7).unwrap().id(), &7);
}
