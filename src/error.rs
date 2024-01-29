use crate::prelude::*;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Error as ReqwestError;
use serde_json::json;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    BadRequest(String),
    InternalServer,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::BadRequest(error) => (StatusCode::BAD_REQUEST, error),
            Error::InternalServer => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        error!("serde_json::Error: {:?}", err);
        Error::InternalServer
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Self {
        error!("ReqwestError: {:?}", err);
        Error::InternalServer
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        error!("IoError: {:?}", err);
        Error::InternalServer
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
