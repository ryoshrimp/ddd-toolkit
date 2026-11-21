use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
#[value_object(no_eq)]
struct Price(f64);

fn assert_eq<T: Eq>() {}
fn assert_hash<T: std::hash::Hash>() {}

fn main() {
    assert_eq::<Price>();
    assert_hash::<Price>();
}
