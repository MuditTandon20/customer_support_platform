use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

// ---- Role enum: mirrors the Postgres `user_role` ENUM type ----
#[derive(Debug, Serialize, Deserialize, Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Agent,
    Customer,
}

// ---- TicketStatus enum: mirrors the Postgres `ticket_status` ENUM type ----
#[derive(Debug, Serialize, Deserialize, Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    New,
    Open,
    Pending,
    OnHold,
    Solved,
    Closed,
}

// ---- Ticket Models & DTOs ----

#[derive(Debug, Serialize)]
pub struct Ticket {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub subject: String,
    pub description: Option<String>,   // NEW
    pub status: TicketStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub description: Option<String>,   // NEW
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketRequest {
    pub status: Option<TicketStatus>,
    pub assigned_to: Option<Uuid>,
}

// ---- Domain events ----
// Each event is self-contained: a reader (Kafka consumer, audit log,
// anything) should never need to query the DB just to understand
// what happened. This is the "domain event" half of the outbox pattern.

#[derive(Debug, Serialize)]
pub struct TicketStatusChanged {
    pub ticket_id: Uuid,
    pub organization_id: Uuid,
    pub old_status: TicketStatus,
    pub new_status: TicketStatus,
    pub changed_by: Uuid,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct TicketAssigned {
    pub ticket_id: Uuid,
    pub organization_id: Uuid,
    pub assigned_to: Option<Uuid>, // None = unassigned
    pub assigned_by: Uuid,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

// ---- User & Auth DTOs ----

#[derive(Deserialize)]
pub struct SignupRequest {
    pub organization_name: Option<String>, // required only when creating a NEW org
    pub organization_id: Option<Uuid>,     // provide this to JOIN an existing org
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: Option<Role>, // ignored when creating a new org (creator is always admin)
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub org_id: Uuid,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: Role,
}
