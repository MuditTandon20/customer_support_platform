ALTER TABLE ai_proposals ADD CONSTRAINT proposal_tenant_id UNIQUE(organization_id,id);
ALTER TABLE messages ADD COLUMN proposal_id UUID;
ALTER TABLE messages ADD CONSTRAINT message_proposal_tenant FOREIGN KEY(organization_id,proposal_id) REFERENCES ai_proposals(organization_id,id);
