use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug)]
pub struct ApiKeyValue(String);

impl ApiKeyValue {
    pub fn new() -> Self {
        Self(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect(),
        )
    }
}
