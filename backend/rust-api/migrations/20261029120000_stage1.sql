CREATE TABLE knowledge_chunks (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL,
 document_id UUID NOT NULL, ordinal INT NOT NULL, content TEXT NOT NULL,
 search_vector TSVECTOR GENERATED ALWAYS AS (to_tsvector('english',content)) STORED,
 FOREIGN KEY(organization_id,document_id) REFERENCES knowledge_documents(organization_id,id) ON DELETE CASCADE,
 UNIQUE(organization_id,document_id,ordinal)
);
CREATE INDEX knowledge_chunk_search ON knowledge_chunks USING GIN(search_vector);
CREATE INDEX knowledge_chunk_tenant ON knowledge_chunks(organization_id,document_id);
CREATE FUNCTION rebuild_knowledge_chunks() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE words TEXT[]; start_at INT; position INT := 0;
BEGIN
 DELETE FROM knowledge_chunks WHERE organization_id=NEW.organization_id AND document_id=NEW.id;
 IF NEW.status='ready' THEN
  words:=regexp_split_to_array(trim(NEW.content),'\s+');
  FOR start_at IN 1..cardinality(words) BY 150 LOOP
   INSERT INTO knowledge_chunks(organization_id,document_id,ordinal,content)
   VALUES(NEW.organization_id,NEW.id,position,array_to_string(words[start_at:least(start_at+179,cardinality(words))],' '));
   position:=position+1;
  END LOOP;
 END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER knowledge_chunk_ingestion AFTER INSERT OR UPDATE OF content,status ON knowledge_documents FOR EACH ROW EXECUTE FUNCTION rebuild_knowledge_chunks();
-- Additive backfill; source content and original document IDs stay intact.
UPDATE knowledge_documents SET content=content;
ALTER TABLE workflow_runs ADD COLUMN trigger_event TEXT NOT NULL DEFAULT 'manual';
CREATE INDEX workflow_daily_usage ON workflow_runs(organization_id,created_at);
CREATE INDEX proposals_daily_usage ON ai_proposals(organization_id,created_at);
CREATE INDEX inbox_updated ON tickets(organization_id,updated_at DESC,id);
CREATE TABLE ai_activity_log (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(), organization_id UUID NOT NULL, ticket_id UUID NOT NULL,
 operation TEXT NOT NULL, mode TEXT NOT NULL, status TEXT NOT NULL, latency_ms BIGINT NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 FOREIGN KEY(organization_id,ticket_id) REFERENCES tickets(organization_id,id)
);
CREATE INDEX ai_activity_tenant ON ai_activity_log(organization_id,created_at DESC);
