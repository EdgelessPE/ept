use rand::{distributions::Alphanumeric, Rng};

pub fn random_short_string() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s
}
