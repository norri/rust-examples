use crate::server::errors::AuthError;
use crate::AppState;
use axum::RequestPartsExt;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::headers::{authorization::Basic, Authorization};
use axum_extra::TypedHeader;
use tracing::{field, Span};

pub struct AuthBasic(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthBasic
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(basic)) =
            parts.extract::<TypedHeader<Authorization<Basic>>>().await?;
        let header_credentials = (basic.username().to_string(), basic.password().to_string());

        let state = AppState::from_ref(state);
        if state.credentials.contains(&header_credentials) {
            // Record the user in the current span
            let span = Span::current();
            span.record("user", field::display(basic.username()));

            Ok(Self(basic.username().to_string()))
        } else {
            Err(AuthError::Failed("credentials not valid".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasources::database::MockDatabase;
    use crate::server::domain::errors::ErrorResponse;
    use crate::server::errors::AppError;
    use crate::test_utils::{init_router, read_response_body, test_authenticated};
    use axum::http::StatusCode;
    use axum::routing::get;
    use base64::Engine;

    async fn test_auth(AuthBasic(_): AuthBasic) -> Result<(), AppError> {
        Ok(())
    }

    #[tokio::test]
    async fn test_valid_credentials() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let header = &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode("user:pass".as_bytes())
        );
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_credentials() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let header = &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode("user:invalid".as_bytes())
        );
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "invalid credentials");
    }

    #[tokio::test]
    async fn test_empty_credentials() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let header = &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(":".as_bytes())
        );
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "invalid credentials");
    }

    #[tokio::test]
    async fn test_missing_authorization_header() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let response = test_authenticated(app, "/protected", "GET", "").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "invalid authentication header");
    }

    #[tokio::test]
    async fn test_malformed_authorization_header() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let header = "Basic malformed_header";
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "invalid authentication header");
    }
}
