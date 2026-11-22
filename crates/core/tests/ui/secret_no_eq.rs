use ddd_toolkit_core::Secret;

fn main() {
    let left = Secret::new(String::from("secret"));
    let right = Secret::new(String::from("secret"));
    let _ = left == right;
}
