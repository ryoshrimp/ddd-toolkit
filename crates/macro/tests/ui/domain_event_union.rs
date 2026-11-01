use ddd_toolkit_macro::DomainEvent;

#[derive(DomainEvent)]
union InvalidEvent {
    value: u64,
}

fn main() {}
