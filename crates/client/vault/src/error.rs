use std::fmt;

/// Why a vault operation failed. Displays as a message for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    InvalidEmail,
    PasswordTooShort,
    WrongPassword,
    NoAccount,
    AccountExists,
    Locked,
    /// The database couldn't be read or written.
    Storage(String),
    /// Stored data couldn't be encrypted, decrypted or decoded.
    Corrupt(String),
}

impl fmt::Display for VaultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEmail => "Enter a valid email address.",
            Self::PasswordTooShort => "The password must be at least 12 characters.",
            Self::WrongPassword => "Wrong password",
            Self::NoAccount => "There is no account on this device.",
            Self::AccountExists => "An account already exists on this device.",
            Self::Locked => "The app is locked.",
            Self::Storage(message) | Self::Corrupt(message) => message,
        })
    }
}

impl std::error::Error for VaultError {}

pub(crate) fn storage(error: impl fmt::Display) -> VaultError {
    VaultError::Storage(error.to_string())
}

pub(crate) fn corrupt(error: impl fmt::Display) -> VaultError {
    VaultError::Corrupt(error.to_string())
}
