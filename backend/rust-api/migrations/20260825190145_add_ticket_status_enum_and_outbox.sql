-- Add migration script here
-- Step 1: Create the ticket_status enum
CREATE TYPE ticket_status AS ENUM ('new', 'open', 'pending', 'on_hold', 'solved', 'closed');

-- Step 2: Map existing 'open' values (the only value used so far) — already valid, no mapping needed.
-- If any other legacy values existed we'd map them here first, same as we did for user_role.

-- Step 3: Drop the old default FIRST, then convert the type, then set the new default
ALTER TABLE tickets
    ALTER COLUMN status DROP DEFAULT;

ALTER TABLE tickets
    ALTER COLUMN status TYPE ticket_status USING status::ticket_status;

ALTER TABLE tickets
    ALTER COLUMN status SET DEFAULT 'new';

-- Step 4: Outbox table — this is what makes event publishing reliable.
-- Whenever a domain event happens, a row goes in here in the SAME
-- transaction as the actual business data change. A separate background
-- process later reads unpublished rows and pushes them to Kafka.
CREATE TABLE outbox_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,        -- e.g. 'TicketStatusChanged', 'TicketAssigned'
    aggregate_id UUID NOT NULL,      -- the ticket_id this event is about
    organization_id UUID NOT NULL,   -- for filtering/debugging, and future multi-tenant Kafka topics
    payload JSONB NOT NULL,          -- the full event data, self-contained
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ         -- NULL until the background publisher successfully sends it to Kafka
);

-- The background publisher will repeatedly query "give me unpublished events" —
-- this index makes that query fast even as the table grows.
CREATE INDEX idx_outbox_unpublished ON outbox_events (created_at) WHERE published_at IS NULL;