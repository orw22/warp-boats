use crate::responses::ErrorResponse;
use log::warn;
use std::convert::Infallible;
use thiserror::Error;
use warp::{
    filters::body::BodyDeserializeError,
    http::StatusCode,
    reject::{self, MethodNotAllowed},
    reply, Rejection, Reply,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database connection error")]
    ConnectionFailed,
    #[error("Resource not found")]
    NotFound,
    #[error("Invalid parameter")]
    InvalidParameter,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("JWT creation error")]
    JWTCreationFailed,
    #[error("Invalid auth header")]
    InvalidAuthHeader,
    #[error("No permission")]
    NoPermission,
    #[error("Missing API key")]
    MissingAPIKey,
    #[error("No credit")]
    NoCredit,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, String::from("Path not found"))
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::ConnectionFailed => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Error::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            Error::InvalidParameter => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::NoPermission => (StatusCode::FORBIDDEN, e.to_string()),
            Error::InvalidCredentials => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::JWTCreationFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            Error::MissingAPIKey => (StatusCode::BAD_REQUEST, e.to_string()),
            Error::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err.find::<BodyDeserializeError>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            String::from("Invalid JSON body or missing field"),
        )
    } else if err.find::<MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            String::from("Method not allowed"),
        )
    } else {
        warn!("Unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Something went wrong"),
        )
    };

    let json = reply::json(&ErrorResponse {
        status: code.to_string(),
        message,
    });
    Ok(reply::with_status(json, code))
}
