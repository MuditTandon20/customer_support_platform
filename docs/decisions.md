# Architecture decisions

1. Retain Rust as authority and Postgres as durable state. Additive migrations only. Keep existing ticket response fields and event types.
2. Runtime SQL queries for new functionality avoid requiring a live production database at compile time. Authentication re-reads membership and session version so demotion/reset invalidates old access.
3. First local retrieval uses PostgreSQL full-text ranking and extractive, cited evidence. It is explicitly labeled deterministic retrieval, with abstention. No paid inference is automatically enabled. Semantic embeddings and model-generated multilingual responses remain separate unverified requirements.
4. Internal notes and draft proposals are distinct from customer-visible messages. A reply is stored as portal-visible; sandbox email is never labeled delivered.
5. Workflow definitions are bounded, validated directed graphs. Rust validates and executes allowlisted local actions; unsupported external actions fail closed. Sensitive tools can be proposed and approved but cannot execute without a provider adapter.
6. Provider integrations default disabled/sandbox. Billing shows local trial entitlements; no charge or checkout without a configured adapter. Do not fabricate revenue/savings or provider delivery.
7. Tests use disposable PostgreSQL only, identified by TEST_DATABASE_URL; never reset the user's existing data.
