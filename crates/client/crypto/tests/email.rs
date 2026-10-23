//! The email address is the account's identifier for key derivation, so the
//! device and the server must agree on its canonical form.

use randomtask_crypto::normalise_email;

#[test]
fn trims_the_email_and_lower_cases_it() {
    assert_eq!(
        normalise_email("  David@Example.COM "),
        Some("david@example.com".to_owned())
    );
}

#[test]
fn refuses_anything_not_shaped_like_an_email_address() {
    for email in [
        "",
        "   ",
        "david",
        "david@",
        "@example.com",
        "david@example",
        "david@.example.com",
        "david@example.",
        "da vid@example.com",
        "david@exam ple.com",
        "david@@example.com",
        "da@vid@example.com",
    ] {
        assert_eq!(normalise_email(email), None, "{email:?}");
    }
}
