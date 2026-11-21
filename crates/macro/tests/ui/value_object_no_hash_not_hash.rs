use ddd_toolkit_macro::ValueObject;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NoHash;

#[derive(ValueObject)]
#[value_object(no_hash)]
struct Wrapped(NoHash);

fn assert_eq<T: Eq>() {}
fn assert_hash<T: std::hash::Hash>() {}

fn main() {
    assert_eq::<Wrapped>();
    assert_hash::<Wrapped>();
}
