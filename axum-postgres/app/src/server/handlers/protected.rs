use crate::server::{
    domain::{common::MessageResponse, errors::ErrorResponse},
    errors::AppError,
    extractors::auth_basic::AuthBasic,
    openapi::PROTECTED_TAG,
};
use axum::Json;

/// Access protected route
///
/// Access a protected route that requires authentication.
#[utoipa::path(
    get,
    path = "/",
    tag = PROTECTED_TAG,
    responses(
        (status = 200, description = "Return hello response", body = MessageResponse ),
        (status = 400, description = "Bad request", body = ErrorResponse, example = json!(ErrorResponse { error: "invalid authentication header".to_string() })),
        (status = 401, description = "Unauthorized to access", body = ErrorResponse, example = json!(ErrorResponse { error: "invalid credentials".to_string() })),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    security(
        ("basic_auth" = [])
    )
)]
pub async fn protected(AuthBasic(user): AuthBasic) -> Result<Json<MessageResponse>, AppError> {
    tracing::info!("User {} accessed protected route", user);

    Ok(Json(MessageResponse {
        message: format!("Hello, {}!", user),
    }))
}

#[cfg(test)]
mod tests {
    use crate::datasources::database::MockDatabase;
    use crate::server::domain::common::MessageResponse;
    use crate::server::handlers::protected::protected;
    use crate::test_utils::{init_router, read_response_body, test_authenticated};
    use axum::http::StatusCode;
    use axum::routing::get;
    use base64::Engine;

    #[tokio::test]
    async fn test_protected() {
        let mock_db = MockDatabase::new();
        let app = init_router(mock_db, "/protected", get(protected)).await;

        let header = &format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode("user:pass".as_bytes())
        );
        let response = test_authenticated(app, "/protected", "GET", header).await;
        assert_eq!(response.status(), StatusCode::OK);

        let response_body: MessageResponse = read_response_body(response).await;
        assert_eq!(response_body.message, "Hello, user!");
    }
}
