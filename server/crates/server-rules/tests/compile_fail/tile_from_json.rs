use server_rules::answer::SafeTile;

fn main() {
    let _: SafeTile = serde_json::from_str("{}").unwrap();
}
