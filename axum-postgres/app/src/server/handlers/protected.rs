use crate::server::domain::todos::ProtectedResponse;
use crate::server::errors::AppError;
use crate::server::extractors::auth_basic::AuthBasic;
use axum::Json;

pub async fn protected(
    AuthBasic((user, _)): AuthBasic,
) -> Result<Json<ProtectedResponse>, AppError> {
    Ok(Json(ProtectedResponse {
        message: format!("Hello, {}!", user),
    }))
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::MockDatabase;
    use crate::server::domain::todos::ProtectedResponse;
    use crate::server::handlers::protected::protected;
    use crate::test_utils::{init_router, read_response_body, test_authenticated};
    use axum::http::StatusCode;
    use axum::routing::get;
    use base64::Engine;

    #[tokio::test]
    async fn test_protected() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, format!("/protected"), get(protected)).await;

        let header = &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode("user:pass".as_bytes())
        );
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::OK);

        let response_body: ProtectedResponse = read_response_body(response).await;
        assert_eq!(response_body.message, "Hello, user!");
    }
}
