//! Regression coverage for serde helper type resolution in an independent consumer.
#![deny(proc_macro_derive_resolution_fallback)]
#![deny(unused_imports)]

mod types {
    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(serde)]
    pub struct Timestamp(i64);

    // These names must not collide with the generated serde helper types.
    pub type Fields = Timestamp;
    pub type Enum = Timestamp;
}

mod consumer {
    use super::types::{Enum, Fields, Timestamp};
    use std::fmt::Debug as ValueDebug;

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(serde)]
    struct Lifetime {
        issued_at: Timestamp,
        expires_at: Fields,
    }

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(serde)]
    struct Window<T: ValueDebug>(Timestamp, T);

    #[derive(ddd_toolkit::ValueObject)]
    #[value_object(serde)]
    enum Status<T: ValueDebug> {
        Named { at: Timestamp, value: T },
        Newtype(Enum),
        Tuple(Timestamp, Fields),
    }

    fn round_trip<T>(value: T, expected: &str)
    where
        T: ddd_toolkit::__private::serde::Serialize
            + for<'de> ddd_toolkit::__private::serde::Deserialize<'de>
            + PartialEq
            + ValueDebug,
    {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, expected);
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value);
    }

    #[test]
    fn imported_types_round_trip_in_all_helper_shapes() {
        round_trip(
            Lifetime::new(Timestamp::new(1), Timestamp::new(2)),
            r#"{"issued_at":1,"expires_at":2}"#,
        );
        round_trip(Window::new(Timestamp::new(1), Timestamp::new(2)), "[1,2]");
        round_trip(
            Status::new_named(Timestamp::new(1), Timestamp::new(2)),
            r#"{"Named":{"at":1,"value":2}}"#,
        );
        round_trip(
            Status::<Timestamp>::new_newtype(Timestamp::new(1)),
            r#"{"Newtype":1}"#,
        );
        round_trip(
            Status::<Timestamp>::new_tuple(Timestamp::new(1), Timestamp::new(2)),
            r#"{"Tuple":[1,2]}"#,
        );
    }
}
