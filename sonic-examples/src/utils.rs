/// Generate a random length string
pub fn random_chars(length: usize, prefix: &str) -> String {
    use rand::{distributions::Alphanumeric, Rng};

    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    format!("{}{}", prefix, suffix)
}

pub fn random_chars_without_prefix(length: usize) -> String {
    random_chars(length, "")
}
