use ddd_toolkit_macro::{Entity, ValueObject};

#[derive(ValueObject)]
#[value_object(validate = check, error = ())]
struct NewField {
    new: u32,
    end: u32,
}

fn check(_: &NewField) -> Result<(), ()> {
    Ok(())
}

#[derive(ValueObject)]
struct TryNewField {
    try_new: u32,
    end: u32,
}

#[derive(Entity)]
#[entity(getters)]
struct PlainEntity {
    #[entity(id)]
    id: u64,
    try_new: u32,
}

fn main() {
    let a = NewField::try_new(1, 2).unwrap();
    assert_eq!(a.new(), &1);
    let b = TryNewField::new(1, 2);
    assert_eq!(b.try_new(), &1);
    let e = PlainEntity { id: 1, try_new: 2 };
    assert_eq!(e.try_new(), &2);
}
