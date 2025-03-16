use std::{borrow::Cow, error::Error};

pub use http::StatusCode;

pub trait ExpectedError: Error {
    fn status(&self) -> StatusCode;
    fn msg(&self) -> Cow<str>;
}
