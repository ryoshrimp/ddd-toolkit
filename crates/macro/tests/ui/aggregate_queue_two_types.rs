use ddd_toolkit_macro::AggregateRoot;

#[derive(AggregateRoot)]
struct TwoTypes {
    #[aggregate(events)]
    events: Vec<u64, u64>,
}

fn main() {}
