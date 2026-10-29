ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT false;
CREATE TABLE identity_tokens (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), user_id UUID NOT NULL REFERENCES users(id), kind TEXT NOT NULL CHECK(kind IN ('reset','verify')),
 token_hash TEXT NOT NULL UNIQUE, expires_at TIMESTAMPTZ NOT NULL DEFAULT now()+interval '1 hour', used_at TIMESTAMPTZ
);
CREATE TABLE sandbox_mail (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id),
 ticket_id UUID, recipient TEXT NOT NULL, subject TEXT NOT NULL, body TEXT NOT NULL,
 direction TEXT NOT NULL CHECK(direction IN ('inbound','outbound','identity')), idempotency_key UUID NOT NULL,
 state TEXT NOT NULL DEFAULT 'sandbox_only', created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 UNIQUE(organization_id,idempotency_key), FOREIGN KEY(organization_id,ticket_id) REFERENCES tickets(organization_id,id)
);
