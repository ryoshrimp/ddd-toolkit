use ddd_toolkit_macro::Entity;

mod domain {
    use super::Entity;

    #[derive(Entity)]
    #[entity(getters)]
    pub struct Account {
        #[entity(id)]
        key: u64,
        #[entity(skip_getter)]
        secret: String,
    }

    #[derive(Entity)]
    pub struct Legacy {
        #[entity(id)]
        id: u64,
        name: String,
    }
}

fn check(account: domain::Account, legacy: domain::Legacy) {
    account.secret();
    account.key();
    legacy.name();
}

fn main() {}
