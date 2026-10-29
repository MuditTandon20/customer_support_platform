# Run locally

## Docker stack

Requirements: Docker Desktop with its Linux engine running and ports 3000/8080 available.

```powershell
.\scripts\init-env.ps1
docker compose up --build
```

Open http://localhost:3000. Create an organization; no payment provider is enabled. Migrations run when Rust starts. AI service and database stay inside the Compose network. `docker compose stop` preserves data. Do not remove the database volume unless intentionally resetting local data.

Optional Kafka broker/publisher (runtime not verified here):

```powershell
docker compose -f compose.yaml -f compose.events.yaml --profile events up --build
```

Optional demo fixtures (synthetic, in a new organization, never mixed into an existing workspace):

```powershell
python -m pip install httpx
python scripts/seed_demo.py
```

The seed script prints the new local demo login. It refuses non-local hosts. To reproduce on the verification stack, set DEMO_API_URL=http://localhost:18080 and DEMO_FRONTEND_ORIGIN=http://localhost:13000.

## Native services

Create a **local** PostgreSQL database, copy backend/rust-api/.env.example to backend/rust-api/.env, and set DATABASE_URL, a random JWT_SECRET of at least 32 characters, and AI_SERVICE_TOKEN. Use the same AI_SERVICE_TOKEN in backend/ai-copilot/.env. Never reuse example secrets in deployed environments.

```powershell
# Terminal 1
cd backend/rust-api
cargo run

# Terminal 2
cd backend/ai-copilot
python -m venv .venv
.venv/Scripts/python.exe -m pip install -r requirements.txt
.venv/Scripts/python.exe -m uvicorn app.main:app --host 127.0.0.1 --port 8001

# Terminal 3
cd frontend
Copy-Item .env.example .env.local # only if no local file exists
npm ci
npm run dev
```

Rust defaults to port 8080, frontend 3000, Python 8001. Browser requests use same-origin /api routes. Set API_PROXY_URL on the Next.js server at runtime (default http://127.0.0.1:8080; Compose uses http://api:8080). The website origin must match FRONTEND_ORIGIN exactly; use localhost consistently. Secure cookies require HTTPS and COOKIE_SECURE=true outside local development. Use HTTPS and a same-site API hostname; SameSite=Strict intentionally excludes cross-site cookies. API bearer clients remain supported.

## Verification without Docker

The test harness uses a unique temporary Postgres directory and localhost:55432. It does not read or reset the user's existing database. Install the test tooling:

```powershell
cd tests
npm ci
npx playwright install chromium
npm run db
```

Run the API against the printed TEST_DATABASE_URL, with API_BIND=127.0.0.1:18080, FRONTEND_ORIGIN=http://localhost:13000, ENABLE_SANDBOX=true and a test JWT secret. Start frontend with API_PROXY_URL=http://127.0.0.1:18080 on port 13000. Then, from the repository root:

```powershell
cargo test --manifest-path backend/rust-api/Cargo.toml --locked
backend/ai-copilot/.venv/Scripts/python.exe -m unittest discover -s tests -p 'test_*.py' -v
python tests/integration.py
cd tests
npm run test:e2e
```

The database harness is test-only. Ctrl+C stops it and leaves the uniquely named directory for inspection. Stop only processes launched for these tests. Test artifacts are in tests/playwright-report and docs/screenshots.

## Failure handling

- API unavailable: inspect `/health`, database readiness, and migrations. SQL errors are logged server-side, omitted from client responses.
- Copilot unavailable: ticket and portal functionality remain usable. Knowledge drafts use local Postgres and do not need an LLM. Summary/classification require the internal Python service.
- Quota reached: local limits are 100 active articles, 200 draft proposals/day, 25 workflows, 500 workflow runs/day; days use UTC. UI billing shows actual counters.
- Outbox: every new ticket/message and ticket update commits with its outbox entry. The local notification projection writes a receipt and notification atomically and retries every two seconds. Kafka publishing is at-least-once, optional via KAFKA_BROKERS. Kafka consumer offsets and broker failover remain unimplemented. Do not treat the local projection as a Kafka consumer.
- Email sandbox: enable only for local development. Identity/reset links are secrets visible to workspace administrators. No email leaves the machine. External email/WhatsApp/provider retries are not implemented.
- Sensitive tools: administrator approval records intent. External writes remain blocked without provider implementation. Never interpret approval as execution.
- Workflows: manual trigger only. Approval is a recorded handoff state; continuation, durable timers, automatic event triggers and persisted failed runs remain incomplete. Database transaction rollback prevents partial local effects on failure.

## Deployment prerequisites

Compose and Docker images are provided but could not be run with this machine's unavailable Docker engine. Container-build validation is still required. Before any real deployment: disable sandbox; provision and validate email/billing adapters and webhooks; enable HTTPS secure cookies; configure trusted ingress/rate limiting; define secret rotation, backups, restore testing, retention and incident ownership; run a database migration rehearsal and load/security/accessibility review. No production deployment, remote push, payment or paid model call is authorized or performed by this build.

Backups must be taken with pg_dump against an explicitly selected database and restored into a separate disposable database. No production backup/restore drill or availability SLO is claimed.
