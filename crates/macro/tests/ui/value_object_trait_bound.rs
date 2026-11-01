use ddd_toolkit_macro::ValueObject;

struct NotClone;

#[derive(ValueObject)]
struct Wrapped(NotClone);

fn main() {
    let value = Wrapped::new(NotClone);
    let _ = value.clone();
}
