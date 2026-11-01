use ddd_toolkit_macro::AggregateRoot;

#[derive(AggregateRoot)]
struct EmptyAttribute {
    #[aggregate()]
    events: Vec<u64>,
}

fn main() {}
