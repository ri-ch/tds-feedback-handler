use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    BoxError, Form, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use axum_sessions::async_session::log::warn;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tower_http::services::ServeDir;
use tracing::debug;
use validator::Validate;

mod appstate;
mod errors;
mod tls;
mod types;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = appstate::AppState::new().await;

    let csrf_config = CsrfConfig::default();

    let inbound_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled error: {}", err),
            )
        }))
        .layer(DefaultBodyLimit::max(1024))
        .layer(BufferLayer::new(1024))
        .layer(RateLimitLayer::new(1, Duration::from_secs(30)));

    let app = Router::new()
        .route_service("/assets", ServeDir::new("./public"))
        .route("/submit", post(insert_record).layer(inbound_layer))
        .route("/", get(home))
        .route("/status", get(status))
        .with_state(state)
        .layer(CsrfLayer::new(csrf_config));

    let port = 8080;

    let message = format!("Running the server on port {}", port);

    debug!(message);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template, Deserialize, Serialize, Validate)]
#[template(path = "index.html")]
struct Page {
    authenticity_token: String,
    #[validate(length(min = 1, max = 100), required)]
    name: Option<String>,
    #[validate(length(min = 1, max = 400), required)]
    message: Option<String>,
}

async fn home(token: CsrfToken) -> impl IntoResponse {
    let page = Page {
        authenticity_token: token.authenticity_token().unwrap(),
        name: None,
        message: None,
    };
    
    (token, page)
}

async fn status() -> &'static str {
    "Service is running"
}

async fn insert_record(
    token: CsrfToken,
    State(state): State<appstate::AppState>,
    Form(page): Form<Page>,
) -> Result<impl IntoResponse, errors::AppError> {
    token.verify(&page.authenticity_token)?;

    if let Err(e) = page.validate() {
        warn!("{}", e);
        return Ok((StatusCode::BAD_REQUEST, "Invalid form values"));
    }

    let client = state.client;

    let insert_statement = client
        .prepare("INSERT INTO feedback_response (name, response, ts) VALUES ($1, $2, $3)")
        .await?;

    let ts = chrono::Local::now();

    let name = page.name.unwrap();
    let message = page.message.unwrap();

    client
        .query(&insert_statement, &[&name, &message, &ts])
        .await?;

    Ok((StatusCode::OK, "OK"))
}
