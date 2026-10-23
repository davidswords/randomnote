use std::fmt;

/// Why a request was refused, with a message written for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    MissingCredentials,
    InvalidEmail,
    UnsupportedVersion,
    InvalidKeyParams,
    EmailTaken,
    InvalidJson,
    /// The database failed. The detail is logged, never shown.
    Storage(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingCredentials => "Please enter an email and a password to register.",
            Self::InvalidEmail => "Enter a valid email address.",
            Self::UnsupportedVersion => "Please update randomtask to register.",
            Self::InvalidKeyParams => "The key parameters are invalid.",
            Self::EmailTaken => "This email is already registered.",
            Self::InvalidJson => "The request is not valid JSON.",
            Self::Storage(_) => "Something went wrong. Please try again.",
        })
    }
}

impl std::error::Error for ServerError {}

pub(crate) fn storage(error: impl fmt::Display) -> ServerError {
    ServerError::Storage(error.to_string())
}
