use rand::Rng;
use rand::distr::Alphanumeric;

pub(crate) fn generate_random_string() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
