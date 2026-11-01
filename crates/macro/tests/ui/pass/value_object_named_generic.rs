use ddd_toolkit_macro::ValueObject;

#[derive(ValueObject)]
struct Named<T>
where
    T: Clone,
{
    value: T,
}

fn main() {
    let value = Named { value: 7_u32 };
    assert_eq!(value.as_inner(), &7_u32);
}
