# Core API contracts

Core base URL: http://localhost:8080. The generated [OpenAPI inventory](../backend/rust-api/openapi.json) lists routes and selected request schemas; it is not a complete response-schema definition. Regenerate with `python scripts/generate_contract.py`. FastAPI exposes its own `/openapi.json` and `/docs` internally.

The website forwards same-origin /api requests through a bounded server-side proxy configured with API_PROXY_URL. It forwards session cookies, authorization and Origin, keeps upstream status codes, and returns a useful 502 when the API is unavailable.

Authenticate browser sessions with the HttpOnly aster_session cookie. Browser fetch uses credentials=include; cookie-authenticated writes require Origin exactly equal to FRONTEND_ORIGIN. Use HTTPS/COOKIE_SECURE=true outside local development. CLI/SDK clients may instead send Authorization: Bearer TOKEN. Signup/login JSON token fields remain compatible with the original API. Browser code stores only a session marker, never the token.

Role checks use live database membership. Cross-tenant and other-customer ticket IDs return the same 404 as absent IDs. Most domain errors are `{ "error": "message" }`; malformed extractors may return Axum text errors. Database/internal details are logged only server-side. Bodies max 256 KiB; auth IP rate limit 30/min, other requests 600/min per process. A trusted production ingress/distributed limiter remains required.

| API family | Permissions and semantics |
|---|---|
| `/auth/signup`, `/auth/login`, `/me` | New org creator is admin. Joining an existing org requires invitation. Signup validates 12–256 character password. |
| `/invitations`, `/auth/accept-invitation` | Admin creates single-use 48-hour agent/customer invitation, matching email required. Manual sharing; no delivery claim. |
| `/auth/request-reset`, `/auth/reset-password`, `/auth/verify-email` | Sandbox-only token delivery. Tokens SHA-256 hashed, single use, one-hour expiry. Reset increments session version. No production mail adapter. |
| `/tickets`, `/tickets/{id}` | Customers create/read own tickets; team reads org tickets. Team changes status/assignment. `unassign:true` clears assignment. Closed is terminal; solved may reopen. |
| `/inbox?q=...` | Subject, description and message search, scoped to org/ownership. Customer search excludes internal notes. Most recent 250. |
| `/tickets/{id}/messages` | GET thread, POST body/internal/idempotency_key/optional proposal_id. Customer cannot post/read notes. Portal delivery is distinct from email. Reuse of a key with different content is rejected. |
| `/tickets/{id}/metadata` | Team priority and labels, max 12 labels and 40 bytes each. |
| `/knowledge`, `/knowledge/{id}/reindex` | Team adds/searches/reindexes; customers see public only. Current sources only. Archive stops retrieval/public listing. Full-text index updates synchronously. |
| `/tickets/{id}/grounded-draft` | Team creates a saved extractive proposal and citations; no evidence means abstention. Sending with proposal_id rechecks source version and freshness. Source wording is untrusted; human must review. |
| `/tools/preview`, `/tools` | Team, strict argument allowlist. Preview has no effects. label/status/lookup are local; refund/cancel/export create approval requests. Org-scoped idempotency. |
| `/approvals`, `/approvals/{id}` | Team lists; admin decides. Approval of external action becomes blocked, executed=false. No sensitive external writes. |
| `/workflows` | Team reads; admin drafts/edits/publishes. 1–32 reachable acyclic nodes. Trigger/condition/action/approval/fallback supported. No durable delay execution. |
| `/workflows/{id}/run` | Team simulates draft; admin executes published version. Ticket ID and key required. DB transaction rolls back partial effects on error. |
| `/workflows/{id}/versions`, `/rollback` | Team reads versions; admin restores historical definition into draft. Must publish explicitly to activate. |
| `/workflow-runs` | Team inspects persisted successful/handoff runs and traces, latest 100. Failed-run persistence not implemented. |
| `/analytics` | Team, real status/volume/response/CSAT/proposal counts with explicit KPI definitions. Resolved status is not independently verified resolution. |
| `/tickets/{id}/csat` | Creator rates own solved/closed ticket from 1 to 5; upsert one rating per ticket. |
| `/settings`, `/users`, `/users/{id}/role` | Team reads members; admin updates settings/roles. Serialized last-admin guard. No arbitrary org switching. |
| `/notifications`, `/notifications/{id}/read` | Team, durable local outbox projection, per-user read receipts. Customer notifications not implemented. |
| `/audit` | Admin reads latest 250 tenant actions. No user deletion/tampering endpoint. Retention not automated. |
| `/billing`, `/integrations` | Actual capability/usage state. No checkout, charges or fabricated invoice records. Trial quotas enforced. |
| `/sandbox/email`, `/sandbox/mailbox` | Administrator plus ENABLE_SANDBOX=true. Simulate known customer's email and see inbound/outbound/identity messages. No external network delivery. |
| `/public/help/{slug}?q=...` | Explicitly published, current public articles only. Anonymous ticket/widget submission is not exposed. |

JavaScript example:

```javascript
const response = await fetch('http://localhost:8080/tickets', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
  body: JSON.stringify({ subject: 'Invoice question', description: 'Please review invoice 42.' })
});
if (!response.ok) throw new Error(await response.text());
const ticket = await response.json();
```

Event compatibility: existing TicketStatusChanged/TicketAssigned payload field names are preserved. New events use `{version:1, organization_id, resource_id, data}`. Outbox UUID is the Kafka record key; duplicate broker delivery is possible. Local projection receipts are unique by (consumer,event_id). There is no implemented external webhook subscription API, scoped API-key issuance, or Kafka-offset consumer; these are remaining PRD requirements.
