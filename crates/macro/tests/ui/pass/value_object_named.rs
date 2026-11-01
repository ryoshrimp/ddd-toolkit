use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct Named {
    value: String,
}

fn main() {
    let value = Named { value: String::from("ok") };
    assert_eq!(value.as_inner(), "ok");
    assert_eq!(Named::new(String::from("new")).into_inner(), "new");
}
