#[derive(Debug)]
pub struct CryptoError(String);

impl std::fmt::Display for CryptoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CryptoError {}

impl CryptoError {
    pub(crate) fn new(message: &str) -> Self {
        Self(message.to_owned())
    }
}

pub(crate) fn to_error(error: impl std::fmt::Display) -> CryptoError {
    CryptoError(error.to_string())
}
