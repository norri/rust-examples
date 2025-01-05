use crate::{datasources::database::DatabaseError, server::domain::errors::ErrorResponse};
use axum::http::StatusCode;
use axum::{
    extract::rejection::JsonRejection,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::typed_header::TypedHeaderRejection;
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound { .. } => AppError::NotFound(error.to_string()),
            DatabaseError::Internal(error) => AppError::Unknown(error),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::AxumJsonRejection(_) => {
                warn!("Invalid JSON in request: {}", self);
                (StatusCode::BAD_REQUEST, "failed to read json".to_string())
            }
            AppError::ValidationError(ref error) => {
                warn!("Validation error: {}", self);
                (StatusCode::BAD_REQUEST, error.to_string())
            }
            AppError::BadRequest(ref message) => {
                warn!("Bad Request: {}", self);
                (StatusCode::BAD_REQUEST, message.clone())
            }
            AppError::NotFound(_) => {
                warn!("Not found: {}", self);
                (StatusCode::NOT_FOUND, "not found".to_string())
            }
            AppError::Unknown(_) => {
                error!("Unknown error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "unknown error".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    HeaderRejection(#[from] TypedHeaderRejection),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::Failed(_) => {
                warn!("Authentication failed: {}", self);
                (StatusCode::UNAUTHORIZED, "invalid credentials".to_string())
            }
            AuthError::HeaderRejection(_) => {
                warn!("Authentication header error: {}", self);
                (
                    StatusCode::BAD_REQUEST,
                    "invalid authentication header".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::read_response_body;
    use anyhow::anyhow;
    use axum::extract::rejection::MissingJsonContentType;
    use axum::response::Response;

    #[tokio::test]
    async fn test_json_extractor_rejection() {
        let json_rejection = MissingJsonContentType::default();
        let app_error = AppError::AxumJsonRejection(json_rejection.into());
        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "failed to read json");
    }

    #[tokio::test]
    async fn test_validation_error() {
        let mut validation_errors = validator::ValidationErrors::new();
        validation_errors.add("text", validator::ValidationError::new("too short"));
        let app_error = AppError::ValidationError(validation_errors);

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(
            response_body.error,
            "text: Validation error: too short [{}]"
        );
    }

    #[tokio::test]
    async fn test_bad_request() {
        let app_error = AppError::BadRequest("too long input".into());

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "too long input");
    }

    #[tokio::test]
    async fn test_not_found() {
        let app_error: AppError = AppError::NotFound("not found".to_string());

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "not found");
    }

    #[tokio::test]
    async fn test_database_not_found() {
        let id = uuid::Uuid::new_v4();
        let db_error = DatabaseError::NotFound { id };
        let app_error: AppError = db_error.into();

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, format!("not found"));
    }

    #[tokio::test]
    async fn test_database_internal_error() {
        let db_error = DatabaseError::Internal(anyhow!("internal error"));
        let app_error: AppError = db_error.into();

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "unknown error");
    }

    #[tokio::test]
    async fn test_unknown() {
        let app_error = AppError::Unknown(anyhow!("unknown error"));

        let response: Response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "unknown error");
    }
}
