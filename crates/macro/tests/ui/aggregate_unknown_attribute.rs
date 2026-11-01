use ddd_toolkit_macro::AggregateRoot;

#[derive(AggregateRoot)]
struct UnknownAttribute {
    #[aggregate(other)]
    events: Vec<u64>,
}

fn main() {}
