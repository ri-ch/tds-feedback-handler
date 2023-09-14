use axum::{http::StatusCode, response::IntoResponse};
use tracing::error;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("An error has occured {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "There was an error processing your request",
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
