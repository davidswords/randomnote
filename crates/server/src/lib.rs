//! The randomtask server: accounts and sessions, following the Standard Notes
//! server's API so that the protocol matches Standard Notes.

mod accounts;
mod database;
mod error;
mod http;
mod registration;
mod session;
mod sign_in;

pub use accounts::{Accounts, Clock, SystemClock};
pub use database::Database;
pub use error::ServerError;
pub use http::router;
