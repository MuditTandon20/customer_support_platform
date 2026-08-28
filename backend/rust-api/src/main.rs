use axum::{
    routing::{get, post, patch},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;

mod auth;
mod outbox;
mod error;
mod models;
mod handlers;

use handlers::{users, tickets};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub ai_copilot_url: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("Connected to Postgres successfully");

    let state = AppState {
        db: pool.clone(),
        http_client: reqwest::Client::new(),
        ai_copilot_url: env::var("AI_COPILOT_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8001".to_string()),
    };

    let kafka_brokers = vec!["localhost:9092".to_string()];
    tokio::spawn(outbox::run_outbox_publisher(pool.clone(), kafka_brokers));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/signup", post(users::signup))
        .route("/auth/login", post(users::login))
        .route("/me", get(users::me))
        .route("/tickets", post(tickets::create_ticket).get(tickets::list_tickets))
        .route("/tickets/:id", get(tickets::get_ticket).patch(tickets::update_ticket))
        .route("/tickets/:id/summary", get(tickets::get_ticket_summary))
        .route("/tickets/:id/category", get(tickets::get_ticket_category))
        .route("/tickets/:id/draft-reply", get(tickets::get_ticket_draft_reply))
        .route("/users/:id/role", patch(users::update_user_role))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}