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
    MissingEmail,
    MissingCodeChallenge,
    /// A sign-in without an email, password or code verifier.
    IncompleteSignIn,
    /// A wrong email, password or code verifier, which are never told apart.
    WrongCredentials,
    /// Too many failed sign-ins in a row for this email.
    Locked,
    MissingRefreshTokens,
    InvalidRefreshToken,
    ExpiredRefreshToken,
    /// A sign-out without a valid access token.
    InvalidSessionToken,
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
            Self::MissingEmail => "Enter your email address.",
            Self::MissingCodeChallenge => "Please provide the code challenge parameter.",
            Self::IncompleteSignIn => "Invalid login credentials.",
            Self::WrongCredentials => "Invalid email or password",
            Self::Locked => "Too many failed sign-ins. Please try again in an hour.",
            Self::MissingRefreshTokens => "Please provide all required parameters.",
            Self::InvalidRefreshToken => "The refresh token is not valid.",
            Self::ExpiredRefreshToken => "The refresh token has expired.",
            Self::InvalidSessionToken => "Invalid session token.",
            Self::Storage(_) => "Something went wrong. Please try again.",
        })
    }
}

impl std::error::Error for ServerError {}

pub(crate) fn storage(error: impl fmt::Display) -> ServerError {
    ServerError::Storage(error.to_string())
}
