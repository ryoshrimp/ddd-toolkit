use ddd_toolkit_macro::ValueObject;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Good;

struct Bad;

#[derive(ValueObject)]
struct Pair {
    good: Good,
    bad: Bad,
}

fn main() {}
