use rand::distr::Alphanumeric;
use rand::Rng;

pub(crate) fn generate_random_string() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
