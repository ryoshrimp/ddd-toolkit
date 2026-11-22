use ddd_toolkit_macro::ValueObject;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Good;

struct Bad;

#[derive(ValueObject)]
enum Choice {
    Ok(Good),
    Broken(Bad),
}

fn main() {}
