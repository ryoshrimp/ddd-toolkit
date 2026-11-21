use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_eq)]
struct Price(f64);

#[derive(ValueObject)]
#[value_object(no_eq, no_hash)]
struct Both {
    amount: f64,
    rate: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NoHash;

#[derive(ValueObject)]
#[value_object(no_hash)]
struct Hashless(NoHash);

fn assert_eq<T: Eq>() {}

#[derive(ValueObject)]
#[value_object(no_eq)]
enum Measure {
    Unset,
    Value(f64),
}

fn assert_traits<T: Clone + PartialEq + std::fmt::Debug>() {}

fn main() {
    assert_traits::<Price>();
    assert_traits::<Both>();
    assert_traits::<Hashless>();
    assert_eq::<Hashless>();
    assert_traits::<Measure>();
}
