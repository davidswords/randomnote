//! The account's email address, which is also its identifier for key
//! derivation, so it's stored in one canonical form.

/// Trims `email` and lower-cases it, or returns None if it isn't shaped like
/// an address: one `@`, a name before it, and a domain with a dot inside it,
/// with no spaces anywhere.
pub(crate) fn normalise(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    let (name, domain) = email.split_once('@')?;
    let valid = !name.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.contains(char::is_whitespace);
    valid.then_some(email)
}
