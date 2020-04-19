use std::error::Error;
use std::fmt;

use reqwest;
use url::ParseError;

#[derive(Debug)]
pub enum ApiClientError {
    HTTPRequestError(String),
    HTTPTimeoutError(reqwest::Error),
    DeSerializerError(String),
    URLParseError(ParseError),
}

impl Error for ApiClientError {}

impl From<ParseError> for ApiClientError {
    fn from(err: ParseError) -> ApiClientError {
        ApiClientError::URLParseError(err)
    }
}

impl From<reqwest::Error> for ApiClientError {
    fn from(err: reqwest::Error) -> ApiClientError {
        ApiClientError::HTTPTimeoutError(err)
    }
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ApiClientError::HTTPRequestError(ref cause) => write!(f, "{}", cause),
            ApiClientError::HTTPTimeoutError(ref err) => err.fmt(f),
            ApiClientError::DeSerializerError(ref cause) => write!(f, "{}", cause),
            ApiClientError::URLParseError(ref err) => err.fmt(f),
        }
    }
}
