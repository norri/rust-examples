use crate::server::errors::AuthError;
use crate::AppState;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use base64::Engine;

#[derive(Debug, Clone, Default, FromRef)]
pub struct AuthBasic(pub (String, String));

#[async_trait]
impl<S> FromRequestParts<S> for AuthBasic
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let authorization_header = get_header(req).map_err(|e| AuthError::Failed(e.to_string()))?;

        let split = authorization_header.split_once(' ');
        match split {
            Some(("Basic", contents)) => {
                let decoded = decode(contents)?;

                let state = AppState::from_ref(state);
                if state.credentials.contains(&decoded) {
                    // TODO Add username to logs
                    Ok(Self(decoded))
                } else {
                    Err(AuthError::Failed("credentials not valid".to_string()))
                }
            }
            _ => Err(AuthError::Failed("invalid header".to_string())),
        }
    }
}

fn decode(input: &str) -> Result<(String, String), AuthError> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| AuthError::Failed(e.to_string()))?;
    let decoded_str =
        String::from_utf8(decoded_bytes).map_err(|e| AuthError::Failed(e.to_string()))?;

    decoded_str
        .split_once(':')
        .map(|(id, password)| (id.to_string(), password.to_string()))
        .ok_or_else(|| AuthError::Failed("invalid credentials".to_string()))
}

pub(crate) fn get_header(parts: &mut Parts) -> Result<&str, String> {
    parts
        .headers
        .get(AUTHORIZATION)
        .ok_or("Authorization header is missing")?
        .to_str()
        .map_err(|e| e.to_string())
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

    async fn test_auth(AuthBasic((_, _)): AuthBasic) -> Result<(), AppError> {
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
    async fn test_missing_authorization_header() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let response = test_authenticated(app, "/protected", "GET", "").await;
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
    async fn test_malformed_authorization_header() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(test_auth)).await;

        let header = "Basic malformed_header";
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response_body: ErrorResponse = read_response_body(response).await;
        assert_eq!(response_body.error, "invalid credentials");
    }
}
