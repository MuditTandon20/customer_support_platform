# Aster Support OS

An evidence-first customer support workspace built on the existing Rust/Axum API, PostgreSQL, Python/FastAPI copilot and Next.js frontend. This build adds a connected local product and preserves the original ticket APIs and event shapes.

The full supplied PRD is **not entirely implemented**. Read [requirements and verification](docs/requirements-traceability.md) for exact status, and [build state](docs/BUILD_STATE.md) for remaining work. External delivery/payment providers are not live. No production-ready claim is made.

## Stage 1 testing release

**Security review completed for the scoped Stage 1 beta.** The reviewed build is eligible for synthetic, non-commercial testing after its CI gates pass. Raw package alerts remain visible with bounded, evidence-backed assessments. Read the [security review closure](docs/STAGE1_SECURITY_CLOSURE.md).

Prepared for a synthetic-data, non-commercial free-tier test. **Not deployed.** See the [release evidence and limitations](docs/STAGE1_RELEASE_REPORT.md), [hosting decision](docs/STAGE1_HOSTING_DECISION.md), and [reproducible setup/future deployment runbook](docs/STAGE1_DEPLOYMENT_RUNBOOK.md). Branch: `release/stage1-testing`. Automatic deployments are disabled. Docker runtime is verified; hosted cold starts remain unverified.

## Run locally

With Docker Desktop's Linux engine running, from the project root:

```powershell
.\scripts\init-env.ps1
docker compose up --build
```

Open **http://localhost:3000** and create an organization. API: http://localhost:8080; its migrations run automatically. No external API key is needed for extractive knowledge drafts. The local stack enables sandbox mail and disables paid inference. The sandbox never sends real email.

Optional synthetic data in a new, explicitly labeled demo organization:

```powershell
python -m pip install httpx
python scripts/seed_demo.py
```

See [the runbook](docs/runbook.md) for native setup, a Docker-free disposable Postgres test harness, environment variables and failure handling. Docker definitions were authored but container execution remains unverified on this machine because the engine is unavailable.

## Connected capabilities

- Dark/light, responsive console with command palette, keyboard access, reduced motion, live operations dashboard and actual event metrics.
- Invitation-only organization membership; administrator/agent/customer roles; HttpOnly browser sessions; origin protection; live role checks and password-reset session revocation.
- Inbox and customer context; threaded portal replies, team-only notes, priority, labels, assignment, ticket state rules and CSAT.
- Current, tenant-scoped knowledge articles; public help center; full-text retrieval; cited extractive proposals, abstention, and source revalidation before proposal dispatch.
- Typed local tools, dry-run previews, idempotency and administrator approval records. Sensitive external execution remains blocked.
- Visual workflow authoring, keyboard reordering and drag/drop; branches, draft/publish/version restoration, simulation, server execution and inspectable traces.
- Audit events, transactional outbox, durable local notification projection, optional Kafka publisher, rate/body limits and safe errors.
- Local email round-trip sandbox, manual invitations, sandbox verification/reset links, honest integration status and enforced trial quotas.
- Authenticated portal widget, customer conversation history and public searchable help center.

## Verification

```powershell
cargo test --manifest-path backend/rust-api/Cargo.toml --locked
python -m unittest discover -s tests -p 'test_*.py' -v
python tests/integration.py
cd frontend
npm ci
npm run lint
npx tsc --noEmit
npm run build
npm audit
cd ../tests
npm ci
npx playwright install chromium
npm run test:e2e
```

HTTP integration/browser tests require the isolated local services described in the runbook. They create synthetic organizations and never reset an existing database. CI definition is in [.github/workflows/ci.yml](.github/workflows/ci.yml). Test results and actual limits are recorded in [the verification report](docs/verification.md).

## Implementation references

- [Original specification](docs/specification.md)
- [Architecture and data flow](docs/architecture.md)
- [API guide](docs/api.md) and [OpenAPI inventory](backend/rust-api/openapi.json)
- [Architecture decisions](docs/decisions.md)
- [Baseline audit](docs/current-state-audit.md)
- [Desktop screenshot](docs/screenshots/workspace-desktop.png)
- [Mobile screenshot](docs/screenshots/workspace-mobile.png)

Not yet implemented: live email/WhatsApp/payments, semantic embeddings and ingestion queues, attachment storage/scanning, granular custom permissions, multiple memberships/org switching, complete workflow durability, production observability/retention/backups and a full developer integration platform. These require further implementation and verification, not merely credentials.

MIT licensed; see [LICENSE](LICENSE).
