CREATE TABLE notifications (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL REFERENCES organizations(id),
 event_id UUID NOT NULL UNIQUE REFERENCES outbox_events(id), resource_id UUID NOT NULL, kind TEXT NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(organization_id,id)
);
CREATE INDEX notification_tenant ON notifications(organization_id,created_at DESC);
CREATE TABLE notification_reads (
 organization_id UUID NOT NULL, notification_id UUID NOT NULL, user_id UUID NOT NULL,
 read_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(organization_id,notification_id,user_id),
 FOREIGN KEY(organization_id,notification_id) REFERENCES notifications(organization_id,id),
 FOREIGN KEY(organization_id,user_id) REFERENCES users(organization_id,id)
);
