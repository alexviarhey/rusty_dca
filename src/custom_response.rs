use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ResultCode {
    Ok = 0,
    Err = 1,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    text: String,
}

impl ResponseMessage {
    pub fn with_text(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomResponse<T> {
    result_code: ResultCode,
    data: Option<T>,
    message: Option<ResponseMessage>,
    validation_errors: Option<HashMap<String, String>>,
}

impl<T> CustomResponse<T> {
    pub fn new(
        result_code: ResultCode,
        data: Option<T>,
        message: Option<ResponseMessage>,
        validation_errors: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            result_code,
            data,
            message,
            validation_errors,
        }
    }
}

impl<T> Default for CustomResponse<T> {
    fn default() -> Self {
        Self::new(ResultCode::Ok, None, None, None)
    }
}

impl<T> IntoResponse for CustomResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json) => (StatusCode::OK, json).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Custom response serelization failed!",
            )
                .into_response(),
        }
    }
}

trait IntoCustomResponse {
    type Data;
    fn into_sustom_response() -> CustomResponse<Self::Data>;
}
