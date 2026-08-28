use rskafka::client::partition::{Compression, UnknownTopicHandling};
use rskafka::client::ClientBuilder;
use rskafka::record::Record;
use sqlx::PgPool;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use uuid::Uuid;

/// One row read out of the outbox_events table.
struct OutboxRow {
    id: Uuid,
    event_type: String,
    payload: serde_json::Value,
}

/// Maps our internal event_type string to a Kafka topic name.
/// Centralizing this mapping here means adding a new event type later
/// only requires one new match arm, not scattered logic everywhere.
fn topic_for_event(event_type: &str) -> &'static str {
    match event_type {
        "TicketStatusChanged" => "ticket.status_changed",
        "TicketAssigned" => "ticket.assigned",
        _ => "ticket.unknown",
    }
}

/// Starts an infinite background loop that:
///   1. Polls the outbox table for unpublished rows
///   2. Publishes each one to its corresponding Kafka topic
///   3. Marks it as published (sets published_at) only after Kafka confirms
///
/// This runs as a separate async task via tokio::spawn, alongside the
/// Axum HTTP server — the two run concurrently, sharing the same DB pool.
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

async fn poll_and_publish(
    pool: &PgPool,
    client: &rskafka::client::Client,
    partition_clients: &mut HashMap<String, rskafka::client::partition::PartitionClient>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = sqlx::query_as!(
        OutboxRow,
        r#"SELECT id, event_type, payload
           FROM outbox_events
           WHERE published_at IS NULL
           ORDER BY created_at ASC
           LIMIT 50"#
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
        sqlx::query!(
            "UPDATE outbox_events SET published_at = now() WHERE id = $1",
            row.id
        )
        .execute(pool)
        .await?;

        published_count += 1;
    }

    Ok(published_count)
}