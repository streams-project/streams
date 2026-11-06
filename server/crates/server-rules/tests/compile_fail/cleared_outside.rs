use server_rules::answer::Cleared;

#[derive(serde::Serialize)]
struct Fake;

impl Cleared for Fake {}

fn main() {}
