use std::{error::Error, fmt, str::from_utf8};

use axum::{body::BoxBody, response::Response};
use http::{HeaderMap, StatusCode};
use hyper::body::to_bytes;
use serde::de::DeserializeOwned;

pub type AssertionError = Box<dyn Error>;
pub type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
pub enum ResponseValidationError {
    DeserializeError(serde_json::error::Error, axum::body::Bytes),
    StatusCode {
        location: String,
        want: StatusCode,
        got: StatusCode,
    },
    Header,
    Body,
}

impl fmt::Display for ResponseValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseValidationError::DeserializeError(err, b) => {
                write!(f, "Deserialization error: {:?}", err.to_string())?;
                write!(f, "bytes: {:?}", from_utf8(b))
            }
            ResponseValidationError::StatusCode {
                location,
                want,
                got,
            } => {
                write!(f, "{location}: want status code {want}, got {got}")
            }
            ResponseValidationError::Header => {
                write!(f, "header validation error")
            }
            ResponseValidationError::Body => {
                write!(f, "body validation error")
            }
        }
    }
}

impl Error for ResponseValidationError {}

pub struct ResponseValidator {
    pub response: Response<axum::body::BoxBody>,
}

impl ResponseValidator {
    #[track_caller]
    pub fn status(self, code: StatusCode) -> Result<Self, ResponseValidationError> {
        if self.response.status() != code {
            let caller_location = std::panic::Location::caller().clone();
            let line = caller_location.line();
            let file = caller_location.file();
            let column = caller_location.column();
            Err(ResponseValidationError::StatusCode {
                location: format!("{file}:{line}:{column}"),
                want: code,
                got: self.response.status(),
            })
        } else {
            Ok(self)
        }
    }
    pub fn header<F>(self, predicate: F) -> Result<Self, ResponseValidationError>
    where
        F: FnOnce(&HeaderMap) -> bool,
    {
        if predicate(&self.response.headers()) {
            Ok(self)
        } else {
            Err(ResponseValidationError::Header)
        }
    }

    pub async fn extract<T: DeserializeOwned>(self) -> Result<T, AssertionError> {
        let body = self.response.into_body();
        let bytes = to_bytes(body).await?;
        serde_json::from_slice(&bytes)
            .map_err(|_err| ResponseValidationError::DeserializeError(_err, bytes).into())
    }

    pub async fn body<T: DeserializeOwned + PartialEq>(
        self,
        want: &T,
    ) -> Result<(), AssertionError> {
        let body = self.response.into_body();
        let bytes = to_bytes(body).await?;
        let body: T = serde_json::from_slice(&bytes)
            .map_err(|_err| ResponseValidationError::DeserializeError(_err, bytes))?;

        if &body != want {
            Err(ResponseValidationError::Body.into())
        } else {
            Ok(())
        }
    }
}

pub trait ResponseExt {
    fn then(self) -> ResponseValidator;
}

impl ResponseExt for http::Response<BoxBody> {
    fn then(self) -> ResponseValidator {
        ResponseValidator { response: self }
    }
}
