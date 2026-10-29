# Aster Stage 1 release evidence

2026-10-29. **ASTER STAGE 1: READY FOR FREE-TIER TEST DEPLOYMENT**, subject to the explicit infrastructure and security limitations below. **Not deployed.** This is a synthetic-data, non-commercial beta, not a production release.

## Delivered behavior and architecture

The existing Rust/Axum API, Python/FastAPI copilot, Next.js/TypeScript frontend and PostgreSQL remain. The browser uses a same-origin Next API proxy; Rust owns authentication, authorization, transactions and persistence. Python is internal and service-token protected. Kafka remains an optional build feature; no broker is required for this beta.

- Organization creation/sign-in, settings, article authoring, customer portal/widget, agent inbox/conversation, reviewed cited replies, customer replies, resolution, CSAT and activity-backed analytics work with persisted PostgreSQL records. Duplicate registration and unavailable-service sign-in errors have browser regression coverage.
- Additive chunk ingestion creates overlapping 180-word chunks (30-word overlap), backfills existing articles, and indexes document/chunk text. Retrieval checks tenant, visibility, archive/freshness and source version. Responses expose document/chunk citations or abstain. Sending a proposal rechecks current sources.
- Clearly labeled deterministic extraction, summary and intent rules require no inference account. AI failures fall back for analysis and leave manual replies available. Staff can inspect mode, outcome and latency in AI activity. Public-beta mode prevents paid inference even when a key is present. No semantic embeddings or generative quality claim is made.
- Published workflows respond synchronously to ticket creation, customer reply, resolution and AI abstention. Local priority/status/label, notification, CSAT and human-review actions are bounded and transactional. A savepoint rolls back failed effects while preserving failure history. Quota-limited runs persist. Approval creates an awaiting-human checkpoint; automatic resumption and durable timers are not implemented.
- Workflow Studio offers templates, visual nodes/connections, drag/reorder, keyboard editing, zoom/scroll pan, versions, preview and persisted execution traces. The navy/blue/teal design supports light/dark themes, mobile navigation, reduced motion, CSS perspective and a lightweight mobile hero. The studio loads separately.
- Server inbox pagination/search/status filters, bounded inputs/pool/timeouts, optional background polling (off), and on-demand notification projection reduce idle resource use. Synthetic-email and organization/member/ticket/message/article limits constrain public testing.

Main implementation: `backend/rust-api/src/{workflow_engine,ai_activity,main,outbox}.rs`, handlers, migration `20261029120000_stage1.sql`, `frontend/src/components`, `frontend/src/app/stage1.css`, Python service, `deploy`, `render.yaml`, Vercel manifests and CI. Full original PRD gaps remain in requirements-traceability.md; this scoped release does not complete billing, external channels or production infrastructure.

## Verification actually executed

| Check | Result |
|---|---|
| Rust tests / formatting | 4 passed; formatting passed |
| Optional Kafka feature | `cargo check --features kafka --locked` passed; no broker runtime tested |
| Python, retrieval, service, contract and release-config tests | 17 passed, including beta paid-inference prevention |
| Existing HTTP integration suites | 11 passed against fresh migrated PostgreSQL |
| Stage 1 HTTP suites | 7 passed: chunk overlap/tail/ACL/staleness, events, rollback/history, reply/CSAT, escalation/logs, pagination, no-poll notifications |
| AI outage profile | All 7 suites passed with Python unavailable; fallback activity assertions passed |
| Restart and beta profile | Native API restart preserved ticket; concurrent organization cap, synthetic-email rejection and public sandbox denial passed |
| Frontend | Lint, TypeScript and production build passed |
| Browser/component/mobile/accessibility | 7 Playwright tests passed; the two component tests passed again after final build. Includes full support journey, dark/light, mobile, reduced motion, axe checks, drag and keyboard editing |
| Dependency audits | Frontend and test npm audit: 0; installed isolated Python OSV: 0; Rust findings remain below |
| Configuration | Compose parsed, release guards passed, generated API contract has 52 paths, Git diff whitespace check passed |
| Container image/start/restart | **UNVERIFIED**: Docker CLI exists but Docker engine is unavailable |
| Hosted providers and Linux execution | **UNVERIFIED locally**; no cloud resource or deployment created. Remote CI status is separate evidence |

One simultaneous load/browser/API run hit Windows socket error 10053; a sequential rerun passed all seven API cases. No automatic retry of uncertain writes was added. A mobile theme control was initially outside the viewport; scrollable navigation fixed it and final browser checks passed. This is useful regression evidence, not a claim of exhaustive security/accessibility testing.

## Measured performance

Production Next.js on local Windows, Playwright Chromium, Lighthouse 12.6.1 mobile simulated throttling. Baseline performance **69**, accessibility **100**, LCP **3079.76 ms**, CLS **0**, TBT **1181.5 ms**. After disabling unnecessary landing-link prefetch, the final run measured performance **81**, accessibility **100**, LCP **2633.38 ms**, CLS **0**, TBT **635 ms**. Both JSON results are retained; two lab runs do not prove a stable improvement or a production target.

The final synthetic login interaction sample recorded 16 ms Event Timing entries and CLS 0. The earlier sample recorded 32 ms and CLS 0.2344 across navigation. **Field p75 INP is UNVERIFIED**: these samples and navigation Lighthouse do not measure it. No traffic/field dataset exists.

The API smoke check made 100 requests, concurrency 10, zero errors: p50 **52.2 ms**, p95 **295.41 ms**, **101.73 requests/s**, in an empty synthetic tenant on Windows debug Rust/Postgres 18. This is not a capacity or availability guarantee.

Local stack readiness: **2065 ms**. Point-in-time working sets: Rust **20.4 MiB**, actual Python worker **59.8 MiB**, Next **83.3 MiB**. Peak working sets were approximately 58.4/59.8/120.1 MiB. These do not establish combined Linux container RSS or free-host memory fit. Static JavaScript physical total was **719,141 bytes** across all route chunks, largest **229,820 bytes** (below the checked 300 KB single-chunk budget). Total is not one page's transfer size.

Evidence: STAGE1_LIGHTHOUSE.json, STAGE1_LIGHTHOUSE_BASELINE.json, STAGE1_INTERACTIONS.json, STAGE1_RESOURCES.json, load-smoke.json, STAGE1_PROFILE_CHECKS.json. Lighthouse is an isolated operator tool outside project dependencies; its dev-tool dependency tree has advisories and is not part of the deployed app. Hosted cold starts, Linux memory, actual throttled free-host load and field Web Vitals remain unmeasured.

## Security and limits

Exact-origin CSRF enforcement covers cookie writes and Origin-bearing bearer requests. HttpOnly/SameSite=Strict sessions, live role/membership checks, ownership checks, hashed reset tokens and revocation, parameterized SQL, action allowlists, source checks and input/rate limits remain. COOKIE_SECURE=true and exact HTTPS FRONTEND_ORIGIN are mandatory later. The public profile rejects non-synthetic email and disables secret-bearing sandbox routes. No real secrets or customer data are included.

SQLx was upgraded to 0.9.0. Linux uses AWS-LC JWT support, removing the unused RustCrypto RSA/bigint graph; native Windows retains RustCrypto for compatibility with the available MSVC environment. **Three active Windows transitive findings remain:** block-buffer 0.10.4 (GHSA-qwgh-2vcv-g2f7), crypto-bigint 0.5.5 (RUSTSEC-2026-0387), rsa 0.9.10 (RUSTSEC-2023-0071). Linux's inspected dependency graph still includes block-buffer through SQLx SHA-2 migration checksums. HS256-only verification reduces exposure to unused RSA operations; it is not an exemption. No clean Rust audit or proven exploitability assessment is claimed. See security.md and dependency-scan.json.

React text rendering and parameterized SQL reduce XSS/SQL-injection exposure; this is source review plus focused negative tests, not penetration testing. URL/file ingestion and executable attachments are absent, so no upload antivirus or arbitrary-URL fetch feature is exposed. Fixed operator-configured internal AI/proxy URLs are not user-selected retrieval URLs. Knowledge text can still contain malicious instructions: extractive output is an excerpt, human review is mandatory, and model text never executes tools. In-memory limits are single-instance only. Fake data only; there is no retention/deletion/export program or hosted backup/restore drill.

## Hosting, costs and future release

Recommendation: **Vercel Hobby + one Render Free combined Rust/Python container + Neon Free**, estimated **₹0/month only within eligible free quotas**. Render sleeps and has 512 MB; Neon scales to zero. Optional Groq/free inference is evaluated but not provisioned or required. No payment methods, paid plans, inference charges or automatic deployments were enabled. Account eligibility, provider restrictions and storage/compute quotas must be rechecked before future deployment. See [hosting decision](STAGE1_HOSTING_DECISION.md) for dated primary sources and alternatives.

Follow [exact setup, future deployment and rollback instructions](STAGE1_DEPLOYMENT_RUNBOOK.md). Container execution, Linux CI and HTTPS smoke checks are required gates before any public launch. Do not auto-upgrade if the free service is inadequate. Scheduled jobs, guaranteed uptime, live mail/payments, hosted backup recovery, semantic vectors and external provider delivery are outside this release.

## Screenshots and Git references

Actual synthetic-account captures: [desktop dashboard](screenshots/demo-dashboard.png), [desktop inbox](screenshots/demo-inbox.png), [workflow studio](screenshots/demo-workflows.png), [mobile dashboard](screenshots/demo-mobile.png), [customer portal](screenshots/customer-portal.png), [light theme](screenshots/workspace-light.png), [mobile sign-in](screenshots/login-mobile.png).

Repository: https://github.com/MuditTandon20/customer_support_platform. Dedicated branch: `release/stage1-testing`. Preserved local-work baseline: `6caa575`; remote main baseline: `d3b4e25`. The Stage 1 commit containing this report is identified by `git log -1 release/stage1-testing`; push/CI evidence is recorded separately at handoff. Main is not merged or force-pushed. The original PDF/HTML/PDF-generator artifacts remain untouched and excluded from release commits.
