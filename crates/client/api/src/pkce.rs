//! The PKCE code verifier and challenge, made as Standard Notes' client makes
//! them, so the server knows the client signing in is the one that asked for
//! the key parameters.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// 256 random bits as 64 lower-case hex characters.
pub(crate) fn code_verifier() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the operating system provides randomness");
    hex::encode(bytes)
}

/// base64url, without padding, of the hex SHA-256 of the verifier.
pub(crate) fn code_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(hex::encode(Sha256::digest(verifier)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_the_challenge_as_standard_notes_does() {
        // Computed independently with Python's hashlib and base64.
        assert_eq!(
            code_challenge("5f1d2c3b4a59687706f5e4d3c2b1a0998877665544332211ffeeddccbbaa0099"),
            "YjcwODBhOGViNzg2ODJiNTc2MDMwMjliNzM3ZjU0MDhiZWFkNjhjNTYxNmVkNjU2Mjc5YjM4MTQyODM2NmJmMQ"
        );
    }
}
