use crate::server::errors::AppError;
use axum::{
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        datasources::database::MockDatabase,
        server::domain::errors::ErrorResponse,
        test_utils::{init_router, read_response_body, test_post},
    };
    use axum::http::StatusCode;
    use axum::routing::post;
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct TestPayload {
        #[validate(length(min = 1, message = "field is required"))]
        field: String,
    }

    async fn test_handler(ValidatedJson(_): ValidatedJson<TestPayload>) -> Result<(), AppError> {
        Ok(())
    }

    #[tokio::test]
    async fn test_valid_json() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/json", post(test_handler)).await;

        let payload = TestPayload {
            field: "value".to_string(),
        };
        let response = test_post(app, "/json", payload).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/json", post(test_handler)).await;

        let payload = TestPayload {
            field: "".to_string(),
        };
        let response = test_post(app, "/json", payload).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "field: field is required");
    }

    #[tokio::test]
    async fn test_malformed_json() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/json", post(test_handler)).await;

        #[derive(Debug, Deserialize, Serialize)]
        struct InvalidPayload {
            field: u32,
        }
        let payload = InvalidPayload { field: 10 };
        let response = test_post(app, "/json", payload).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "failed to read json");
    }
}
