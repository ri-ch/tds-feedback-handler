use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    BoxError, Form, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use tracing::{debug, warn};
use validator::Validate;

mod appstate;
mod errors;
mod tls;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
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
        // Note: global rate limit, not per-IP. Use tower-governor for per-IP limiting.
        .layer(RateLimitLayer::new(5, Duration::from_secs(1)));

    let app = Router::new()
        .route("/submit", post(insert_record).layer(inbound_layer))
        .route("/", get(home))
        .route("/status", get(status))
        .with_state(state)
        .layer(CsrfLayer::new(csrf_config));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

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
) -> Result<Response, errors::AppError> {
    token.verify(&page.authenticity_token)?;

    if let Err(e) = page.validate() {
        warn!("{}", e);
        return Ok((StatusCode::BAD_REQUEST, e.to_string()).into_response());
    }

    let client = state.pool.get().await?;

    let insert_statement = client
        .prepare("INSERT INTO feedback_response (name, response, ts) VALUES ($1, $2, $3)")
        .await?;

    let ts = chrono::Local::now();

    let name = page.name.expect("name validated as required");
    let message = page.message.expect("message validated as required");

    client
        .query(&insert_statement, &[&name, &message, &ts])
        .await?;

    Ok((StatusCode::OK, "OK").into_response())
}
