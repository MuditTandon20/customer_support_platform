#[cfg(feature = "kafka")]
use rskafka::client::partition::{Compression, UnknownTopicHandling};
#[cfg(feature = "kafka")]
use rskafka::client::ClientBuilder;
#[cfg(feature = "kafka")]
use rskafka::record::Record;
use sqlx::PgPool;
#[cfg(feature = "kafka")]
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use uuid::Uuid;

/// One row read out of the outbox_events table.
#[cfg(feature = "kafka")]
#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    event_type: String,
    payload: serde_json::Value,
}

/// Maps our internal event_type string to a Kafka topic name.
/// Centralizing this mapping here means adding a new event type later
/// only requires one new match arm, not scattered logic everywhere.
#[cfg(feature = "kafka")]
fn topic_for_event(event_type: &str) -> &'static str {
    match event_type {
        "TicketStatusChanged" => "ticket.status_changed",
        "TicketAssigned" => "ticket.assigned",
        "MessageCreated" => "conversation.message_created",
        "TicketCreated" => "ticket.created",
        _ => "ticket.unknown",
    }
}

/// Local durable projection; each receipt and notification commits atomically.
/// This does not claim to be a Kafka consumer or replace Kafka offsets.
pub async fn run_notification_projection(pool: PgPool) {
    loop {
        if let Err(e) = project_for_org(&pool, None).await {
            tracing::error!(error=%e,"Notification projection failed; retrying");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
pub async fn project_for_org(pool: &PgPool, org: Option<Uuid>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let events:Vec<(Uuid,Uuid,Uuid,String)>=sqlx::query_as("SELECT e.id,e.organization_id,e.aggregate_id,e.event_type FROM outbox_events e WHERE ($1::uuid IS NULL OR e.organization_id=$1) AND NOT EXISTS(SELECT 1 FROM consumer_receipts r WHERE r.event_id=e.id AND r.consumer='notifications-v1') ORDER BY e.created_at LIMIT 50 FOR UPDATE OF e SKIP LOCKED").bind(org).fetch_all(&mut *tx).await?;
    for (event_id, org_id, resource, kind) in events {
        sqlx::query("INSERT INTO notifications(organization_id,event_id,resource_id,kind) VALUES($1,$2,$3,$4) ON CONFLICT(event_id) DO NOTHING").bind(org_id).bind(event_id).bind(resource).bind(kind).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO consumer_receipts(consumer,event_id) VALUES('notifications-v1',$1) ON CONFLICT DO NOTHING").bind(event_id).execute(&mut *tx).await?;
    }
    tx.commit().await
}

/// Starts an infinite background loop that:
///   1. Polls the outbox table for unpublished rows
///   2. Publishes each one to its corresponding Kafka topic
///   3. Marks it as published (sets published_at) only after Kafka confirms
///
/// This runs as a separate async task via tokio::spawn, alongside the
/// Axum HTTP server — the two run concurrently, sharing the same DB pool.
#[cfg(feature = "kafka")]
pub async fn run_outbox_publisher(pool: PgPool, kafka_brokers: Vec<String>) {
    // Give Kafka a moment if the container just started.
    let client = loop {
        match ClientBuilder::new(kafka_brokers.clone()).build().await {
            Ok(c) => break c,
            Err(e) => {
                eprintln!("Kafka client connection failed, retrying in 3s: {e}");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };

    println!("Outbox publisher connected to Kafka, starting poll loop");

    // Cache one PartitionClient per topic so we don't reconnect every poll.
    let mut partition_clients = HashMap::new();

    loop {
        match poll_and_publish(&pool, &client, &mut partition_clients).await {
            Ok(count) if count > 0 => {
                println!("Outbox publisher: published {count} event(s)");
            }
            Ok(_) => {} // nothing to publish this cycle, stay quiet
            Err(e) => {
                eprintln!("Outbox publisher error: {e}");
            }
        }

        // Poll every 2 seconds. A real production system might use
        // Postgres LISTEN/NOTIFY to react instantly instead of polling,
        // but polling is simpler to reason about and plenty fast for learning.
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(feature = "kafka")]
async fn poll_and_publish(
    pool: &PgPool,
    client: &rskafka::client::Client,
    partition_clients: &mut HashMap<String, rskafka::client::partition::PartitionClient>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = sqlx::query_as::<_, OutboxRow>(
        r#"SELECT id, event_type, payload
           FROM outbox_events
           WHERE published_at IS NULL
           ORDER BY created_at ASC
           LIMIT 50"#,
    )
    .fetch_all(pool)
    .await?;

    let mut published_count = 0;

    for row in rows {
        let topic = topic_for_event(&row.event_type);

        // Lazily create (and cache) a PartitionClient for this topic.
        if !partition_clients.contains_key(topic) {
            let pc = client
                .partition_client(topic, 0, UnknownTopicHandling::Retry)
                .await?;
            partition_clients.insert(topic.to_string(), pc);
        }
        let partition_client = partition_clients.get(topic).unwrap();

        let record = Record {
            key: Some(row.id.to_string().into_bytes()),
            value: Some(serde_json::to_vec(&row.payload)?),
            headers: BTreeMap::new(),
            timestamp: chrono::Utc::now(),
        };

        partition_client
            .produce(vec![record], Compression::NoCompression)
            .await?;

        // Only mark as published AFTER Kafka has confirmed receipt.
        // If the process crashed between publish and this UPDATE, the
        // row would simply be re-published next cycle — at-least-once
        // delivery, which is why consumers should treat events as
        // idempotent (safe to process more than once).
        sqlx::query("UPDATE outbox_events SET published_at = now() WHERE id = $1")
            .bind(row.id)
            .execute(pool)
            .await?;

        published_count += 1;
    }

    Ok(published_count)
}
