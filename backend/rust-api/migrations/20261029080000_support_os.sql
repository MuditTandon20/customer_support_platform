ALTER TABLE users ADD COLUMN token_version BIGINT NOT NULL DEFAULT 0;
ALTER TABLE organizations ADD COLUMN settings JSONB NOT NULL DEFAULT '{}';
ALTER TABLE tickets ADD COLUMN priority TEXT NOT NULL DEFAULT 'normal' CHECK (priority IN ('low','normal','high','urgent'));
ALTER TABLE tickets ADD COLUMN labels TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE tickets ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE tickets ADD COLUMN resolved_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN channel TEXT NOT NULL DEFAULT 'portal';
ALTER TABLE tickets ADD CONSTRAINT tickets_tenant_id UNIQUE (organization_id,id);
ALTER TABLE users ADD CONSTRAINT users_tenant_id UNIQUE (organization_id,id);
-- Guard new writes without rewriting historical assignments.
ALTER TABLE tickets ADD CONSTRAINT ticket_creator_tenant FOREIGN KEY (organization_id,created_by) REFERENCES users(organization_id,id) NOT VALID;
ALTER TABLE tickets ADD CONSTRAINT ticket_assignee_tenant FOREIGN KEY (organization_id,assigned_to) REFERENCES users(organization_id,id) NOT VALID;

CREATE TABLE messages (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id),
 ticket_id UUID NOT NULL, author_id UUID NOT NULL, body TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 20000),
 internal BOOLEAN NOT NULL DEFAULT false, delivery TEXT NOT NULL DEFAULT 'portal_visible' CHECK (delivery IN ('portal_visible','internal')),
 idempotency_key UUID NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 FOREIGN KEY (organization_id,ticket_id) REFERENCES tickets(organization_id,id),
 FOREIGN KEY (organization_id,author_id) REFERENCES users(organization_id,id), UNIQUE(organization_id,idempotency_key)
);
CREATE INDEX messages_thread ON messages(organization_id,ticket_id,created_at);
CREATE TABLE knowledge_documents (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id),
 title TEXT NOT NULL, content TEXT NOT NULL, visibility TEXT NOT NULL CHECK (visibility IN ('team','public')),
 status TEXT NOT NULL DEFAULT 'ready' CHECK(status IN ('ready','archived')), version INT NOT NULL DEFAULT 1,
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), expires_at TIMESTAMPTZ,
 search_vector TSVECTOR GENERATED ALWAYS AS (to_tsvector('english',title || ' ' || content)) STORED,
 UNIQUE(organization_id,id)
);
CREATE INDEX knowledge_search ON knowledge_documents USING GIN(search_vector);
CREATE INDEX knowledge_tenant ON knowledge_documents(organization_id,status);
CREATE TABLE audit_events (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id),
 actor_id UUID, action TEXT NOT NULL, resource_id UUID, detail JSONB NOT NULL DEFAULT '{}', created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX audit_tenant_time ON audit_events(organization_id,created_at DESC);
CREATE TABLE ai_proposals (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id), ticket_id UUID NOT NULL,
 draft TEXT NOT NULL, citations JSONB NOT NULL, abstained BOOLEAN NOT NULL, mode TEXT NOT NULL DEFAULT 'extractive',
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), FOREIGN KEY (organization_id,ticket_id) REFERENCES tickets(organization_id,id)
);
CREATE TABLE tool_executions (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id), ticket_id UUID NOT NULL,
 requested_by UUID NOT NULL, approved_by UUID, action TEXT NOT NULL, arguments JSONB NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('pending','approved','rejected','executed','blocked')), result JSONB NOT NULL DEFAULT '{}',
 idempotency_key UUID NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 FOREIGN KEY (organization_id,ticket_id) REFERENCES tickets(organization_id,id), UNIQUE(organization_id,idempotency_key)
);
CREATE TABLE workflows (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id), name TEXT NOT NULL,
 draft JSONB NOT NULL, published JSONB, version INT NOT NULL DEFAULT 0, updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(organization_id,id)
);
CREATE TABLE workflow_versions (
 organization_id UUID NOT NULL, workflow_id UUID NOT NULL, version INT NOT NULL, definition JSONB NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(organization_id,workflow_id,version),
 FOREIGN KEY(organization_id,workflow_id) REFERENCES workflows(organization_id,id)
);
CREATE TABLE workflow_runs (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL, workflow_id UUID NOT NULL, ticket_id UUID NOT NULL,
 version INT NOT NULL, simulation BOOLEAN NOT NULL, state TEXT NOT NULL, trace JSONB NOT NULL,
 idempotency_key UUID NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 FOREIGN KEY(organization_id,workflow_id) REFERENCES workflows(organization_id,id),
 FOREIGN KEY(organization_id,ticket_id) REFERENCES tickets(organization_id,id), UNIQUE(organization_id,idempotency_key)
);
CREATE TABLE invitations (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id), email TEXT NOT NULL,
 role user_role NOT NULL CHECK(role IN ('agent','customer')), token_hash TEXT NOT NULL UNIQUE, used_at TIMESTAMPTZ,
 expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + interval '48 hours'
);
CREATE TABLE csat (
 organization_id UUID NOT NULL, ticket_id UUID NOT NULL, customer_id UUID NOT NULL, score INT NOT NULL CHECK(score BETWEEN 1 AND 5),
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(organization_id,ticket_id),
 FOREIGN KEY(organization_id,ticket_id) REFERENCES tickets(organization_id,id)
);
CREATE TABLE consumer_receipts (
 consumer TEXT NOT NULL, event_id UUID NOT NULL REFERENCES outbox_events(id), processed_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(consumer,event_id)
);
