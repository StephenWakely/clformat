//! An error struct that makes it easier for us to report the relevant errors while parsing.
use std::ops::Deref;

use nom::error::{FromExternalError, ParseError};

#[derive(Clone, Debug)]
pub(crate) struct FormatError<I> {
    input: I,
    error: ErrorType,
}

#[derive(Clone, Debug)]
enum ErrorType {
    /// The error has come from Nom. Ideally we shouldn't get to the stage where we report
    /// these errors.
    Nom,
    /// The error has come from us wrapping the parser with `map_res` to define our own error
    /// message.
    Ours(String),
}

impl<T> FormatError<T>
where
    T: Deref<Target = str>,
{
    /// Returns the position in the input string that this error starts.
    /// Assumes the the input string in the error message is the string from the point
    /// the error occurred up to the end of the format string.
    /// Not used until I can actually work out how to make use of it in a span...
    #[allow(unused)]
    pub(crate) fn error_pos(&self, input: T) -> usize {
        input.deref().len() - self.input.deref().len()
    }
}

impl<I> ParseError<I> for FormatError<I> {
    fn from_error_kind(input: I, _kind: nom::error::ErrorKind) -> Self {
        Self {
            input,
            error: ErrorType::Nom,
        }
    }

    fn append(input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        Self {
            input,
            error: other.error,
        }
    }
}

impl<I, E> FromExternalError<I, E> for FormatError<I>
where
    E: ToString,
{
    fn from_external_error(input: I, _kind: nom::error::ErrorKind, e: E) -> Self {
        Self {
            input,
            error: ErrorType::Ours(e.to_string()),
        }
    }
}

impl<I> ToString for FormatError<I>
where
    I: Deref<Target = str>,
{
    fn to_string(&self) -> String {
        match &self.error {
            ErrorType::Nom => "internal error".to_string(),
            ErrorType::Ours(error) => error.to_string(),
        }
    }
}
