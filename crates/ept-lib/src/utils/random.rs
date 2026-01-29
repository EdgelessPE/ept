use rand::{distr::Alphanumeric, Rng};

pub fn random_short_string() -> String {
    let s: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s
}
