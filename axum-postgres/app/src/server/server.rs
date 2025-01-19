use super::routes::new_router;
use crate::SharedState;
use axum::Router;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

pub const TODO_TAG: &str = "Todos";
pub const PROTECTED_TAG: &str = "Protected";

#[derive(OpenApi)]
#[openapi(
    info(title = "Axum Postgres Todos API", description = "Todos Api description"),
    modifiers(&SecurityAddon),
    tags(
        (name = TODO_TAG, description = "Todos API"),
        (name = PROTECTED_TAG, description = "Protected API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "basic_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Basic)),
            )
        }
    }
}

pub fn new_server(app_state: SharedState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(new_router(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::datasources::database::{Database, MockDatabase};
    use crate::test_utils::test_get;
    use crate::AppState;

    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_new_server() {
        let app_state = Arc::new(AppState {
            db: Database::Mock(MockDatabase::new()),
            credentials: vec![("user".to_string(), "password".to_string())],
        });
        let app = new_server(app_state);

        let response = test_get(app, "/swagger-ui").await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }
}
