use ddd_toolkit_macro::AggregateRoot;

#[derive(AggregateRoot)]
struct NoTypeArgument {
    #[aggregate(events)]
    events: Vec,
}

fn main() {}
