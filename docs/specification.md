# PRD: AI-Native Customer Support OS | Full Platform Build
Version 2.0 | Status: implementation specification | Scope: complete integrated product, not a 30-day MVP plan

## 1. Vision and target customer
Build a premium, multi-tenant AI-first customer support SaaS for small and growing B2B SaaS companies (initial ideal customer: 5–50 staff). Consolidate support channels, enable evidence-grounded AI responses, safely automate authorized tasks, and demonstrate verified resolutions and costs. Do not position generic chatbots as differentiation. Distinguish verified resolution, explainable actions, low setup friction, and strong tenant isolation.

## 2. Existing code: preserve and verify
The expected baseline includes Rust/Axum, PostgreSQL, JWT/Argon2, organization-level RBAC, ticket state machine, transactional outbox, Kafka/rskafka, idempotent consumer with offsets, Next.js, and Python/FastAPI AI service. This is background, NOT proof of implementation. Codex must inspect actual files, migrations, tests and history. Keep working APIs and data, avoid a rewrite, and use additive migrations. Preserve ticket state transitions and 404 behavior for inaccessible cross-tenant resources.

## 3. Product modules: implement the complete connected experience
### 3.1 Identity, organizations, billing and onboarding
Signup/login, invitation acceptance, password reset, email verification, secure sessions, admin/agent/customer roles, granular permissions, org switching only where authorized, organization branding, onboarding wizard, sample/demo workspace clearly labeled, usage quotas, plan enforcement, Stripe or compatible payment provider abstraction with webhook verification, subscriptions, invoices and trial controls. Never silently enable paid integrations.

### 3.2 Omnichannel conversation hub
Tickets and conversations unified under customer identity; website widget and email ingestion/sending first, WhatsApp adapter only with approved credentials and official provider; conversation threading, assignment, internal notes, attachments with malware/size checks, labels, priority, status, SLA, collision detection, canned replies, search/filter, bulk actions, keyboard shortcuts, mentions and notification center. Idempotent inbound webhook ingestion, delivery retry and provenance tracking. No fake channel integrations.

### 3.3 AI copilot and knowledge intelligence
Tenant-scoped knowledge sources: files, URLs, FAQs, curated help articles and resolved tickets with explicit opt-in. Ingestion queue, deduplication, chunking, embeddings, indexing, reindexing, document permissions, freshness metadata, citations linking to permitted evidence, hybrid search where useful, grounded answer draft, refusal/abstention when evidence missing, language detection and multilingual response, tone settings, summarization, intent/category/urgency classification, duplicate suggestions, suggested routing, reply quality review. AI suggestions are visually distinct from sent messages.

### 3.4 Safe agentic resolution
Tool registry with strict typed schemas, per-tenant connection permissions, allowlisted read actions (account lookup, order status, subscription status) and selected low-risk writes (ticket labels/status) with policy enforcement. High-impact actions (refunds, cancellations, account changes, payments, data export) require explicit authorized human approval; do not implement autonomous sensitive writes. Include dry-run preview, approval queue, execution trace, idempotency keys, timeouts, retry policy, rate limits, compensating action notes, tool audit and safe escalation. Treat retrieved content as untrusted; never let prompt text override tool authorization.

### 3.5 Visual workflow studio
Accessible node-based editor with trigger, condition, branch, action, approval, delay and fallback nodes. Drag/drop, pan/zoom, minimap, keyboard alternative, versioning, draft/publish, validation, dry-run simulator, execution history, failure inspection, templates and rollback. Enforce bounded execution, cycle detection, permissions and per-org quotas. Server owns workflow execution; client visualization is not the execution engine.

### 3.6 Analytics and AI evaluation
Live operations dashboard, backlog, SLA breaches, response/handling times, agent workload, CSAT survey, deflection estimates separated from confirmed resolution, human takeover rate, citation coverage, hallucination/unsupported answer flags, model latency, token cost, per-tenant budgets, trends and export. Include reproducible eval datasets with tenant-safe synthetic fixtures, retrieval relevance, groundedness, tool correctness, escalation accuracy and regression gates. Label estimates, never present fabricated savings as verified ROI.

### 3.7 Customer-facing experiences
Embeddable chat widget, self-service portal, searchable knowledge center, ticket status/history, authenticated customer messages, satisfaction survey, branded email notifications and responsive mobile experience. Public widget APIs need rate limits, abuse prevention, CSRF/CORS controls and no access to internal notes.

### 3.8 Admin, security and reliability
RBAC and org isolation at every layer, audit trail, retention controls, data export/deletion, encrypted secrets, redaction, signed webhooks, secure attachment storage, PII protections, backups and restore drills, observability with OpenTelemetry, health checks, metrics, traces, alerts, retries, dead-letter queue, outbox consistency, consumer idempotency, incident runbooks and feature flags. Do not claim compliance certification without evidence.

### 3.9 Developer platform and integrations
Versioned REST/OpenAPI docs, scoped API keys, webhook subscriptions, integration settings, secret rotation, sandbox/test events, integration status pages and documented SDK examples. Integrations should have working mocks/sandboxes if credentials unavailable, clearly labeled as such.

## 4. UI/UX specification: premium, functional, accessible
Design language: elegant dark-first enterprise console with optional light theme, slate/ink neutrals, restrained electric violet/cyan accents, readable typography, consistent 8px spacing scale, strong contrast and subtle depth. Avoid excessive blur, illegible glass panels, ornamental gradients or fake analytics.

Core screens: marketing landing page, sign-in/signup, onboarding, dashboard, unified inbox, ticket detail with AI copilot, customer profiles, knowledge base and ingestion, workflow builder, agent approvals, analytics and evals, integrations, billing, org/team/settings, audit logs, customer portal and embedded widget. Every screen needs responsive desktop/tablet/mobile states, loading skeletons, error/empty/success states, meaningful form validation and real wired actions.

Motion: Framer Motion or equivalent for page transitions, staggered list entrances, hover/press microinteractions, animated status changes, optimistic feedback, charts and workflow edges. Prefer transform/opacity animations (150–300 ms), avoid layout jank, respect prefers-reduced-motion, never animate continuously without purpose. Keyboard command palette (Cmd/Ctrl+K), shortcuts, focus management, ARIA labels, accessible drag/drop alternatives, toast notifications, confirmation dialogs for destructive actions. Target WCAG 2.2 AA.

## 5. System architecture
Frontend: Next.js/TypeScript, Tailwind + accessible component library, motion library, TanStack Query or equivalent, typed API client. Rust/Axum is authoritative for users, tenants, tickets, workflows and billing state. PostgreSQL with additive migrations, pgvector where practical; Redis optional for cache/rate limiting. Python/FastAPI hosts retrieval, model adapters, inference and evaluation. Kafka/outbox/consumer retained if already functional; use reliable async jobs and dead-letter handling. Object storage for attachments. Instrument all services. Prefer an extensible modular architecture over unnecessary microservice sprawl.

Request flow: inbound message -> authenticated/validated tenant context -> durable conversation event + outbox -> processing -> tenant-safe retrieval -> draft or permitted tool plan -> approval when required -> dispatch -> customer-visible result -> audit + metrics. Distinguish suggested, attempted, sent, and verified-resolved states.

## 6. Data model and contracts
Core entities: Organization, User, Membership, Role/Permission, Customer, Conversation, Message, Ticket, Assignment, SLA, Attachment, KnowledgeSource, KnowledgeDocument, Chunk/Embedding, RetrievalCitation, AIInvocation, AIProposal, ToolConnection, ToolExecution, Approval, WorkflowDefinition/Version/Run, Integration, WebhookDelivery, AuditEvent, UsageMeter, Subscription, CSAT, EvaluationCase/Run, OutboxEvent and ConsumerOffset. Add schema only after inspecting existing tables. Include indexes, tenant foreign keys, lifecycle state diagrams, retention and uniqueness/idempotency constraints. Publish API contracts and event schemas with compatibility tests.

## 7. Functional acceptance criteria
- Two organizations cannot read, search, retrieve vectors, execute tools against, or receive notifications about each other's records; negative tests across every boundary.
- Customer can create a conversation through web widget, agent sees it in inbox, AI suggests cited answer, human approves and sends, customer sees response, audit and metrics update.
- Email channel round trip works with configured provider; without credentials, show a clearly labeled sandbox rather than pretend success.
- Knowledge ingestion supports errors, retries, deletion and reindexing; unauthorized or stale sources cannot be cited.
- Tool execution requires scoped authorization and approval for sensitive operations; duplicate requests cannot duplicate side effects.
- Workflow can be authored, validated, simulated, published, run and inspected with bounded retries and audit.
- Analytics match underlying events and show source/definition for each KPI; AI costs and limits are enforced.
- Subscription webhook signatures, plan entitlements and usage meters are tested; no accidental charges.
- Every page and action has meaningful loading/error/empty states; responsive and keyboard accessible.
- Tests: unit, integration, contract, end-to-end, security, accessibility, basic load and AI eval regression. No mocked success represented as production success.

## 8. Nonfunctional goals and validation
Propose measurable SLOs after baseline load tests: API availability, p95 latency, queue lag, event processing correctness, AI time-to-first-draft, ingestion throughput and cost per resolved case. Instrument first; document actual measured results and environment. Use rate limiting, backpressure, quotas, budget caps, retries, circuit breakers and graceful fallback to human-only mode.

## 9. Complete-build delivery contract (not a calendar roadmap)
Codex should execute all modules as one coordinated implementation effort, using internal dependency order and continuous integration, not stopping after a first slice. Before edits: inspect repo, record baseline, create traceability matrix and architecture decision log. Then implement across backend, frontend, AI, integrations, tests, docs and deployability. Run build, lint, typecheck, unit/integration/e2e and security checks; fix regressions. Deliver a functioning local Docker Compose stack, example environment configuration, database migrations, seeded demo data clearly marked, documented setup and deploy instructions, API docs, architecture diagrams and a final requirement-by-requirement verification report. If a feature requires external credentials, paid services, privileged access or deployment permission, implement adapters and a verifiable local sandbox, label it blocked, and request approval instead of fabricating completion. Do not push, deploy, spend money, destroy data or alter production without explicit authorization.

## 10. Definition of done
A feature is done only when backend and UI are integrated, tests pass, security and tenant isolation are verified, accessibility states exist, errors are handled, docs are updated and behavior is demonstrable. Final report must classify each requirement as verified complete / implemented but unverified / blocked / not implemented with evidence. Never claim the entire platform is complete merely because code was generated.
