use ddd_toolkit_macro::AggregateRoot;

#[derive(AggregateRoot)]
struct AttributeValue {
    #[aggregate(events = true)]
    events: Vec<u64>,
}

fn main() {}
