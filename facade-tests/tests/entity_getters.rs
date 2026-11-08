use ddd_toolkit::Entity;

mod domain {
    use super::Entity;

    pub struct NonClone(pub u32);

    #[derive(Entity)]
    #[entity(getters)]
    pub struct Account<T> {
        #[entity(id)]
        key: u64,
        name: String,
        payload: T,
        r#type: bool,
        #[entity(skip_getter)]
        secret: String,
    }

    impl<T> Account<T> {
        pub fn new(payload: T) -> Self {
            Self {
                key: 7,
                name: "Ada".into(),
                payload,
                r#type: true,
                secret: "hidden".into(),
            }
        }

        // A skipped field can retain a handwritten accessor with different semantics.
        pub fn secret(&self) -> bool {
            !self.secret.is_empty()
        }
    }

    #[derive(Entity)]
    #[entity(getters, validate = check, error = &'static str, serde)]
    pub struct Stock {
        #[entity(id, skip_getter)]
        id: u64,
        quantity: u32,
        #[entity(skip_getter)]
        secret: String,
    }

    fn check(stock: &Stock) -> Result<(), &'static str> {
        if *stock.quantity() > 0 {
            Ok(())
        } else {
            Err("empty")
        }
    }

    #[derive(Entity)]
    pub struct Manual {
        #[entity(id)]
        id: u64,
        value: u32,
    }

    impl Manual {
        pub fn new() -> Self {
            Self { id: 1, value: 2 }
        }
        // No opt-in: existing handwritten getters must not conflict.
        pub fn value(&self) -> u32 {
            self.value
        }
    }
}

#[test]
fn getters_borrow_generic_non_clone_fields_and_keep_identity_accessor() {
    let account = domain::Account::new(domain::NonClone(42));
    let name: &String = account.name();
    let payload: &domain::NonClone = account.payload();
    assert_eq!(name, "Ada");
    assert_eq!(payload.0, 42);
    assert!(std::ptr::eq(payload, account.payload()));
    assert_eq!(account.r#type(), &true);
    assert_eq!(account.id(), &7);
    assert!(account.secret());
}

#[test]
fn validation_and_id_skip_preserve_existing_identity_accessor() {
    assert!(domain::Stock::try_new(1, 0, "hidden".into()).is_err());
    let stock = domain::Stock::try_new(1, 3, "hidden".into()).unwrap();
    assert_eq!(stock.quantity(), &3);
    assert_eq!(stock.id(), &1);
}

#[test]
fn getter_exclusion_does_not_change_serde_or_validation() {
    let json = serde_json::json!({ "id": 1, "quantity": 3, "secret": "hidden" });
    let stock: domain::Stock = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(stock.quantity(), &3);
    assert_eq!(serde_json::to_value(stock).unwrap(), json);
    assert!(
        serde_json::from_value::<domain::Stock>(serde_json::json!({
            "id": 1, "quantity": 0, "secret": "hidden"
        }))
        .is_err()
    );
}

#[test]
fn no_opt_in_preserves_handwritten_getters() {
    assert_eq!(domain::Manual::new().value(), 2);
}
