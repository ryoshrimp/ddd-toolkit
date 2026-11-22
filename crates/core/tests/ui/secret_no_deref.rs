use ddd_toolkit_core::secret::Secret;

fn main() {
    let secret = Secret::new(String::from("secret"));
    let _: &str = &secret;
}
