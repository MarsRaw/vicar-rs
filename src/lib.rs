#[macro_use]
extern crate lazy_static;

/// Formats an error object to a string via {:?} Debug derived method
macro_rules! t {
    ($error_message:expr) => {
        format!("{:?}", $error_message)
    };
}

pub mod pvl;
pub mod vicar;
