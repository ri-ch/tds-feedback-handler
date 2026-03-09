use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Form, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use serde::{Deserialize, Serialize};
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tracing::{debug, warn};
use validator::Validate;

mod appstate;
mod errors;
mod tls;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = appstate::AppState::new().await?;

    let csrf_config = CsrfConfig::default();

    let governor_conf: &'static _ = Box::leak(Box::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(5)
            .burst_size(10)
            .finish()
            .expect("Invalid governor configuration"),
    ));

    let app = Router::new()
        .route(
            "/submit",
            post(insert_record).layer(DefaultBodyLimit::max(1024)),
        )
        .route("/", get(home))
        .route("/status", get(status))
        .with_state(state)
        .layer(CsrfLayer::new(csrf_config))
        .layer(GovernorLayer { config: governor_conf });

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let message = format!("Running the server on port {}", port);

    debug!(message);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await?;
    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
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

async fn home(token: CsrfToken) -> Result<impl IntoResponse, errors::AppError> {
    let page = Page {
        authenticity_token: token.authenticity_token()?,
        name: None,
        message: None,
    };

    Ok((token, page))
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
        .prepare_cached("INSERT INTO feedback_response (name, response, ts) VALUES ($1, $2, $3)")
        .await?;

    let ts = chrono::Utc::now();

    let name = page.name.expect("name validated as required");
    let message = page.message.expect("message validated as required");

    client
        .query(&insert_statement, &[&name, &message, &ts])
        .await?;

    Ok((StatusCode::OK, "OK").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/", get(home))
            .route("/status", get(status))
            .layer(CsrfLayer::new(CsrfConfig::default()))
    }

    #[tokio::test]
    async fn status_returns_ok() {
        let response = test_router()
            .oneshot(Request::builder().uri("/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn home_returns_ok() {
        let response = test_router()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn page_rejects_none_name() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: None,
            message: Some("hello".to_string()),
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_rejects_empty_name() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("".to_string()),
            message: Some("hello".to_string()),
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_rejects_name_too_long() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("a".repeat(101)),
            message: Some("hello".to_string()),
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_rejects_none_message() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("Alice".to_string()),
            message: None,
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_rejects_empty_message() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("Alice".to_string()),
            message: Some("".to_string()),
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_rejects_message_too_long() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("Alice".to_string()),
            message: Some("a".repeat(401)),
        };
        assert!(page.validate().is_err());
    }

    #[test]
    fn page_accepts_valid_input() {
        let page = Page {
            authenticity_token: "t".to_string(),
            name: Some("Alice".to_string()),
            message: Some("Hello, world!".to_string()),
        };
        assert!(page.validate().is_ok());
    }
}
