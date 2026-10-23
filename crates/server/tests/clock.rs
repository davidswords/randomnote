//! The server's clock against the real system time.

use std::time::{SystemTime, UNIX_EPOCH};

use randomtask_server::{Clock, SystemClock};

#[test]
fn tells_the_time_in_milliseconds_since_1970() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let now = SystemClock.now();

    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    assert!(
        (before..=after).contains(&(now as u128)),
        "{before} {now} {after}"
    );
}
