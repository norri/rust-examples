use super::openapi::new_openapi_router;
use crate::{
    server::handlers::{protected, todos_create, todos_delete, todos_list, todos_update},
    SharedState,
};
use axum::{
    http::{HeaderName, Request},
    routing::get,
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info_span};
use utoipa_axum::{router::OpenApiRouter, routes};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn new_router(app_state: SharedState) -> Router {
    new_openapi_router().merge(add_routes(app_state))
}

fn add_routes(app_state: SharedState) -> OpenApiRouter {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    let request_id = req.headers().get(REQUEST_ID_HEADER);
                    let method = req.method();
                    let uri = req.uri();
                    // user is added if the request is authenticated
                    let user = tracing::field::Empty;

                    if let Some(request_id) = request_id {
                        info_span!("http_request", request_id = ?request_id, %method, %uri, user)
                    } else {
                        error!("could not extract request_id");
                        info_span!("http_request", %method, %uri, user)
                    }
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(TimeoutLayer::new(Duration::from_secs(30)));

    let todos_api_routes = OpenApiRouter::new()
        .routes(routes!(todos_list::todos_list, todos_create::todos_create))
        .routes(routes!(
            todos_update::todos_update,
            todos_delete::todos_delete
        ));

    let protected_routes = OpenApiRouter::new().routes(routes!(protected::protected));

    OpenApiRouter::new()
        .route("/status", get(|| async { "OK" }))
        .nest("/api/v1/todos", todos_api_routes)
        .nest("/api/v1/protected", protected_routes)
        .layer(middleware)
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasources::database::{Database, MockDatabase};
    use crate::test_utils::test_get;
    use crate::AppState;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::Router;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_status_endpoint() {
        let app_state = Arc::new(AppState {
            db: Database::Mock(MockDatabase::new()),
            credentials: vec![("user".to_string(), "password".to_string())],
        });
        let app: Router = add_routes(app_state).into();

        let response = test_get(app, "/status").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "OK");
    }
}
