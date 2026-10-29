mod jwt_crypto;
use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use axum::{
    routing::{get, patch, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::cors::CorsLayer;

mod ai_activity;
mod auth;
mod error;
mod handlers;
mod models;
mod outbox;
mod policy;
mod workflow_engine;

use handlers::{identity, platform, sandbox, tickets, users};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub ai_copilot_url: String,
    pub rate_limits: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    assert!(
        env::var("JWT_SECRET").unwrap_or_default().len() >= 32,
        "JWT_SECRET needs at least 32 characters"
    );

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(
            env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
        )
        .min_connections(0)
        .idle_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(15))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("Connected to Postgres successfully");

    // Automatically run any pending migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Database migrations ran successfully");

    let state = AppState {
        db: pool.clone(),
        http_client: reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap(),
        ai_copilot_url: env::var("AI_COPILOT_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8001".to_string()),
        rate_limits: Arc::new(Mutex::new(HashMap::new())),
    };
    if env::var("BACKGROUND_PROCESSING").as_deref() == Ok("true") {
        tokio::spawn(outbox::run_notification_projection(pool.clone()));
    }

    #[cfg(feature = "kafka")]
    if let Ok(brokers) = env::var("KAFKA_BROKERS") {
        tokio::spawn(outbox::run_outbox_publisher(
            pool.clone(),
            brokers.split(',').map(String::from).collect(),
        ));
    }

    let origin: HeaderValue = env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".into())
        .parse()
        .expect("Invalid FRONTEND_ORIGIN");
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(health_check))
        .route(
            "/live",
            get(|| async { axum::Json(serde_json::json!({"status":"alive"})) }),
        )
        .route("/release", get(release_info))
        .route("/ai/activity", get(platform::ai_activity))
        .route("/openapi.json", get(api_contract))
        .route("/auth/signup", post(users::signup))
        .route("/auth/login", post(users::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/request-reset", post(identity::request_reset))
        .route("/auth/reset-password", post(identity::reset))
        .route("/auth/verify-email", post(identity::verify))
        .route("/sandbox/mailbox", get(sandbox::mailbox))
        .route("/sandbox/email", post(sandbox::inbound))
        .route("/auth/accept-invitation", post(platform::accept_invite))
        .route("/me", get(users::me))
        .route(
            "/tickets",
            post(tickets::create_ticket).get(tickets::list_tickets),
        )
        .route(
            "/tickets/:id",
            get(tickets::get_ticket).patch(tickets::update_ticket),
        )
        .route("/tickets/:id/summary", get(tickets::get_ticket_summary))
        .route("/tickets/:id/category", get(tickets::get_ticket_category))
        .route(
            "/tickets/:id/draft-reply",
            get(tickets::get_ticket_draft_reply),
        )
        .route("/users", get(users::list_users))
        .route("/users/:id/role", patch(users::update_user_role))
        .route("/inbox", get(platform::inbox))
        .route(
            "/tickets/:id/messages",
            get(platform::messages).post(platform::reply),
        )
        .route("/tickets/:id/metadata", patch(platform::metadata))
        .route("/tickets/:id/grounded-draft", post(platform::grounded))
        .route("/tickets/:id/csat", post(platform::csat))
        .route(
            "/knowledge",
            get(platform::knowledge).post(platform::add_knowledge),
        )
        .route(
            "/knowledge/:id",
            axum::routing::delete(platform::delete_knowledge),
        )
        .route("/knowledge/:id/reindex", post(platform::reindex_knowledge))
        .route("/knowledge/:id/chunks", get(platform::chunks))
        .route("/tools/preview", post(platform::tool_preview))
        .route("/tools", post(platform::request_tool))
        .route("/approvals", get(platform::approvals))
        .route("/approvals/:id", post(platform::decide))
        .route(
            "/workflows",
            get(platform::workflows).post(platform::save_workflow),
        )
        .route("/workflows/:id", patch(platform::edit_workflow))
        .route("/workflows/:id/versions", get(platform::versions))
        .route("/workflows/:id/rollback", post(platform::rollback))
        .route("/workflows/:id/publish", post(platform::publish_workflow))
        .route("/workflows/:id/run", post(platform::run_workflow))
        .route("/workflow-runs", get(platform::runs))
        .route("/analytics", get(platform::analytics))
        .route("/audit", get(platform::audit_log))
        .route("/notifications", get(platform::notifications))
        .route("/notifications/:id/read", post(platform::read_notification))
        .route(
            "/settings",
            get(platform::settings).patch(platform::update_settings),
        )
        .route("/integrations", get(platform::integrations))
        .route("/billing", get(platform::billing))
        .route("/invitations", post(platform::invite))
        .route("/public/help/:slug", get(platform::public_help))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit))
        .layer(cors)
        .with_state(state);

    let address = env::var("API_BIND").unwrap_or_else(|_| {
        format!(
            "0.0.0.0:{}",
            env::var("PORT").unwrap_or_else(|_| "8080".into())
        )
    });
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();

    tracing::info!(address=%address,"Server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Signal handler");
        tokio::select! {_ = tokio::signal::ctrl_c()=>{},_ = terminate.recv()=>{}}
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    tracing::info!("Graceful shutdown requested");
}
async fn release_info() -> axum::Json<serde_json::Value> {
    axum::Json(
        serde_json::json!({"stage":"Stage 1 testing beta","commercial":false,"ai_mode":"deterministic extractive","scheduled_workflows":false,"synthetic_emails_required":env::var("BETA_MODE").as_deref()==Ok("true")}),
    )
}

async fn health_check(State(s): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&s.db).await {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({"status":"healthy","service":"rust-api"})),
        ),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"status":"unavailable"})),
        ),
    }
}
async fn api_contract() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        include_str!("../openapi.json"),
    )
}

async fn rate_limit(State(s): State<AppState>, request: Request, next: Next) -> Response {
    let cookie_auth =
        request.headers().contains_key("cookie") || request.headers().contains_key("origin");
    if cookie_auth
        && !matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        )
    {
        let expected =
            env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
        if request
            .headers()
            .get("origin")
            .and_then(|v| v.to_str().ok())
            != Some(expected.as_str())
        {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({"error":"Invalid request origin"})),
            )
                .into_response();
        }
    }
    let ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|v| v.0.ip().to_string())
        .unwrap_or_else(|| "local".into());
    let auth = request.uri().path().starts_with("/auth/");
    let key = format!("{ip}:{auth}");
    let allowed = {
        let mut rates = s.rate_limits.lock().unwrap();
        rates.retain(|_, (at, _)| at.elapsed() < Duration::from_secs(60));
        let entry = rates.entry(key).or_insert((Instant::now(), 0));
        entry.1 += 1;
        entry.1 <= if auth { 30 } else { 600 }
    };
    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({"error":"Request limit reached; retry in 60 seconds"})),
        )
            .into_response();
    }
    next.run(request).await
}
