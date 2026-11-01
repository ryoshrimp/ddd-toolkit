use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
enum Generic<'a, T> {
    Empty,
    Borrowed(&'a T),
    Named { value: T },
}

fn main() {
    let value = Generic::new_named(3_u32);
    assert!(value.is_named());
    assert_eq!(value.as_named().unwrap(), (&3,));
}
