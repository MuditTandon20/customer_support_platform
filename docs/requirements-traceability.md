> Stage 1 adds chunk ingestion/retrieval, bounded event-triggered workflows, persisted failure traces, AI activity, beta limits and hosting preparation. See [the scoped release matrix and evidence](STAGE1_RELEASE_REPORT.md). Historical full-PRD gaps below still apply unless explicitly superseded there.

# Requirement-by-requirement verification

Date: 2026-10-29. Source: [supplied PRD](specification.md). The entire PRD is **not complete**. Status applies only to the capability named in each row, not to its entire parent module.

VERIFIED = implemented and demonstrated by the referenced local tests. IMPLEMENTED-UNVERIFIED = code/configuration exists but the target runtime has not been exercised. BLOCKED = external access/runtime is unavailable in addition to any stated implementation gap. NOT IMPLEMENTED = absent; credentials alone will not complete it.

| PRD | Capability | Status | Implementation / evidence |
|---|---|---|---|
| 3.1 | Signup/login, admin/agent/customer, live roles, tenant ownership, last-admin guard | VERIFIED | auth.rs, handlers/users.rs; integration 01/02/12; Rust JWT/legacy-hash tests |
| 3.1 | Single-use matching-email invitations | VERIFIED | handlers/platform.rs; integration fixture and browser/customer.spec.ts |
| 3.1 | HttpOnly sessions, cookie-origin protection, reset revocation | VERIFIED | auth.rs, main.rs, handlers/identity.rs; integration 12 |
| 3.1 | Password reset and verification token flow in local mailbox | VERIFIED (sandbox) | identity.rs, EmailSandbox.tsx, AccountRecovery.tsx; reset and single-use verification integration (email delivery not live) |
| 3.1 | Organization name/tone/onboarding preferences, trial counters/quotas | VERIFIED preferences/counters; IMPLEMENTED-UNVERIFIED quota exhaustion | settings/billing APIs and screens; browser module checks, API quota code; full quota exhaustion tests not performed |
| 3.1 | Granular custom permissions, multiple memberships/org switching, subscriptions/invoices/trial expiry | NOT IMPLEMENTED | Current model retains one organization per user. Local trial counters only. |
| 3.2 | Conversations, assignment/unassignment, notes, labels, priority, state transitions, search | VERIFIED | tickets.rs, platform.rs, Conversation.tsx; integration 02/03/08/11 and browser flow |
| 3.2 | Email inbound ? inbox ? reply ? outbound mailbox | VERIFIED (sandbox) | handlers/sandbox.rs; integration 09; no external email sent |
| 3.2 | Live email, WhatsApp, signed inbound provider webhooks/delivery retry | NOT IMPLEMENTED / external credentials also BLOCKED | No production provider adapter. Status screens state not configured. |
| 3.2 | Attachment storage/malware checks, mentions, canned replies, bulk actions, SLAs, collision presence | NOT IMPLEMENTED | Row locking prevents transaction races; there is no presence/collision UX or SLA engine. |
| 3.3 | Curated articles, team/public visibility, freshness, archive/reindex, lexical retrieval | VERIFIED | Postgres generated search index; integration 04; knowledge screen |
| 3.3 | Saved cited extractive draft, abstention, permission/freshness/version recheck on dispatch | VERIFIED | grounded/reply handlers, Conversation.tsx; integration 04; browser/customer flow |
| 3.3 | Local summary/classification, service authentication | VERIFIED | FastAPI local_engine.py and routers; integration 04 and test_service.py |
| 3.3 | File/URL ingestion queue, dedup/chunking/embeddings, semantic/hybrid ranking, source ACLs beyond team/public | NOT IMPLEMENTED | No URL fetcher, object store or vector index is claimed. |
| 3.3 | Multilingual generation, calibrated classification, duplicates/routing/quality review, paid-model budget/cost metering | NOT IMPLEMENTED | Language is not detected. Local rules are explicitly uncalibrated. Paid inference defaults off. Tone preference is stored, not applied to extractive quotations. |
| 3.4 | Typed local tool registry, preview, approval queue, trace/audit, idempotency | VERIFIED | platform.rs tool handlers; integration 05/11; approvals and conversation UI |
| 3.4 | Sensitive external execution | BLOCKED / adapter NOT IMPLEMENTED | Refund/cancellation/export requests require admin; approval records blocked/executed=false. No external side effect. |
| 3.4 | External connection scopes, compensation, retries/circuit breakers | NOT IMPLEMENTED | No external execution engine. HTTP client has timeout; this is not a complete retry policy. |
| 3.5 | Visual graph editing, drag/drop and keyboard reorder, branches, bounded acyclic validation | VERIFIED graph policy/studio flow; IMPLEMENTED-UNVERIFIED drag/reorder interactions | policy.rs, WorkflowStudio.tsx; Rust graph tests; browser studio and mobile checks |
| 3.5 | Draft/publish/history, version restore, simulation, manual server run, run idempotency | VERIFIED | workflow tables/handlers; integration 06/11; browser flow. Restore updates draft only. |
| 3.5 | Durable delay, automatic event triggers, pause/resume, retry/failure persistence, minimap | NOT IMPLEMENTED | Approval node stops with awaiting_human; no automatic continuation. Failed transactions roll back but failed runs are not stored. |
| 3.6 | Actual backlog/status/volume/first reply/CSAT/citation counts, JSON export | VERIFIED metrics; IMPLEMENTED-UNVERIFIED browser file download | analytics handler and screens; integration 07/08; browser analytics |
| 3.6 | Synthetic retrieval/abstention/tool-boundary regression fixtures | VERIFIED | 7 retrieval tests, policy tests and HTTP tool/tenant checks; not a model-groundedness benchmark |
| 3.6 | Verified resolution, deflection/human takeover, hallucination flags, token cost/latency and ROI | NOT IMPLEMENTED | Agent-set solved is explicitly not verified resolution. No fabricated savings. |
| 3.7 | Authenticated customer ticket/reply/history/CSAT, searchable public help center | VERIFIED | customer.spec.ts; integration ownership/CSAT/public knowledge |
| 3.7 | Embeddable authenticated portal launcher | VERIFIED (local) | /widget; customer browser checks; launches secure portal in another tab |
| 3.7 | Anonymous chat transport, widget abuse/CSRF design for anonymous sessions, branded external mail | NOT IMPLEMENTED | No anonymous messaging API is exposed. |
| 3.8 | Tenant checks, audit/outbox transactions, durable idempotent local notification projection | VERIFIED | migrations, handlers, outbox.rs; integration 02?11 |
| 3.8 | Kafka publishing retained | IMPLEMENTED-UNVERIFIED | rskafka publisher retained/configurable; broker profile authored. Broker unavailable here. |
| 3.8 | Kafka offset consumer/DLQ, OTel traces/metrics/alerts, encrypted provider secrets | NOT IMPLEMENTED | Local notification receipts are not broker offsets. Request logs/readiness are implemented. |
| 3.8 | Retention/export/deletion, PII redaction policy, backup/restore drills, production security/SLOs | NOT IMPLEMENTED | Runbook outlines prerequisites; no execution evidence/certification claimed. |
| 3.9 | Typed frontend client, OpenAPI endpoint inventory/core schemas, SDK example, honest integration status | VERIFIED | lib/platform.ts, api-client.ts, /openapi.json, docs/api.md, test_contract.py |
| 3.9 | Scoped API keys, webhook subscriptions/delivery/retry, secret rotation, versioned endpoint aliases | NOT IMPLEMENTED | Existing unversioned paths retained. Contract response schemas incomplete. |
| UI | Marketing/auth/workspace screens, dark/light, responsive/keyboard/reduced motion/command palette | VERIFIED (tested screens) | Playwright and axe: dashboard/login/customer conversation; mobile scans inbox/knowledge/workflows/customers/approvals/integrations/billing/settings/audit. Full WCAG 2.2 AA certification not claimed. |
| UI | Error/loading/empty/status handling | VERIFIED (principal flows) | WorkspaceUI.tsx and API client; not every provider failure can be exercised without adapters. |
| Delivery | Native integrated stack, migrations and labeled demo seed | VERIFIED | Disposable Postgres, Rust, Python and Next run locally; scripts/seed_demo.py |
| Delivery | Dockerfiles/Compose configuration | IMPLEMENTED-UNVERIFIED; runtime BLOCKED | Compose config passes, but Docker engine is unavailable. Container builds not executed. |
| Delivery | CI workflow | IMPLEMENTED-UNVERIFIED | .github/workflows/ci.yml authored; no remote push or hosted CI execution performed. |
| Security | Npm dependency audit | VERIFIED | Next 16.4.3 and transitive updates; npm audit zero findings at verification time |
| Security | Rust/Python dependency review | VERIFIED scan, unresolved findings remain | Python isolated dependency scan has no reported findings; three Rust transitive advisories remain. See security.md and dependency-scan.json. |

See [verification.md](verification.md) for exact commands/results, [security.md](security.md) for dependency caveats, and [BUILD_STATE.md](BUILD_STATE.md) for continuation instructions.
