use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub(crate) type ApiResult<T> = Result<Json<T>, ApiError>;

pub(crate) struct ApiError(String);

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl From<String> for ApiError {
    fn from(error: String) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: self.0 }),
        )
            .into_response()
    }
}
