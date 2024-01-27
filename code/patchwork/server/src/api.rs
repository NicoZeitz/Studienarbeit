use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{any, post},
    Router,
};

pub fn api_router() -> Router {
    Router::new()
        .route("/upi", post(|| async { "TODO:" }))
        .fallback_service(any(not_found))
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
