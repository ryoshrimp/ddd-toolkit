use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct Two(String, String);

#[derive(ValueObject)]
struct Named {
    left: u32,
    right: u32,
}

fn main() {
    let tuple = Two::new(String::from("a"), String::from("b"));
    assert_eq!(tuple.field_0(), "a");
    assert_eq!(tuple.field_1(), "b");
    let _: &Two = tuple.as_inner();

    let named = Named::new(1, 2);
    assert_eq!(named.left(), &1);
    assert_eq!(named.right(), &2);
    let _: Named = named.into_inner();
}
