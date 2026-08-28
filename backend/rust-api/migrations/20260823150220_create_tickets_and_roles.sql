-- Add migration script here
-- Step 1: Create the enum type (including 'owner' if needed)
CREATE TYPE user_role AS ENUM ('admin', 'agent', 'customer', 'owner');

-- Step 2: Map old string values to valid enum values
UPDATE users SET role = 'customer' WHERE role = 'member';

-- Step 3: Drop old default, convert type, then set new default
ALTER TABLE users ALTER COLUMN role DROP DEFAULT;
ALTER TABLE users ALTER COLUMN role TYPE user_role USING role::user_role;
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'customer'::user_role;

-- Step 4: Tickets table
CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    subject TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tickets_organization_id ON tickets(organization_id);
CREATE INDEX idx_tickets_created_by ON tickets(created_by);
