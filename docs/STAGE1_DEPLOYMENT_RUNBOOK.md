# Future deployment runbook — do not execute without later authorization

**Security gate:** read STAGE1_SECURITY_CLOSURE.md. The current scoped beta assessment resolves the earlier hold; require green integrated/container/security CI before launch. These instructions do not authorize deployment or claim a clean package scan.

This release is preparation only. No cloud resources or deployment were created. Use an entirely new synthetic-data database and free accounts; never point this beta at a real customer database. Hosting research and cost boundaries are in STAGE1_HOSTING_DECISION.md.

## Reproduce the verified native environment

Use Rust 1.97+, Node 22 and Python 3.10+. Install the Python requirements into backend/ai-copilot/.venv. Use `npm ci` in frontend and tests. A C compiler is needed for ring on both native platforms. JWT uses the supported HS256-only ring adapter on Windows and Linux; the unused RSA/ECC provider graph and AWS-LC build configuration were removed.

```powershell
cargo test --manifest-path backend/rust-api/Cargo.toml --locked
cargo build --manifest-path backend/rust-api/Cargo.toml --locked
cd frontend
npm ci
npm run lint
npx tsc --noEmit
npm run build
cd ../tests
npm ci
npx playwright install chromium
node run-stage1.mjs
```

The harness starts a unique disposable PostgreSQL cluster on 55433, Rust on 18080, Python on 18001 and the production Next server on 13000. It creates no cloud resources and does not reset an existing database. Keep this terminal running. In another terminal, from repository root:

```powershell
backend/ai-copilot/.venv/Scripts/python.exe -m unittest discover -s tests -p 'test_*.py' -v
backend/ai-copilot/.venv/Scripts/python.exe tests/integration.py
backend/ai-copilot/.venv/Scripts/python.exe tests/stage1_integration.py
backend/ai-copilot/.venv/Scripts/python.exe scripts/check_bundle.py
cd tests
node profile-checks.mjs
npm run test:e2e
```

Ctrl+C stops the processes launched by this harness; its temporary database directory is retained for inspection. Existing local test databases and the original user's files are preserved. Port conflicts must be resolved before starting a second harness.

## Future cloud steps

1. **Check authorization and free quotas again.** Confirm the release commit and successful CI. Do not add a payment method, turn on paid plans, or accept automatic upgrades. If any essential free resource is unavailable, stop and use native/local testing.
2. **Neon:** manually create a fresh Free project after authorization; choose a supported Postgres version and a region near Frankfurt. Copy the provider's pooled TLS URL into secret DATABASE_URL with certificate verification. Keep the free compute scale-to-zero behavior enabled. Budget at most 500 MB storage conservatively and monitor compute/egress. pgvector is supported but is not required by this release.
3. **Render:** create one Docker web service from `release/stage1-testing`, at an explicitly reviewed commit. Use `deploy/Dockerfile.stage1`, repository-root context, **Free** plan and Frankfurt. Keep automatic deploys off. `render.yaml` describes this but is not proof of execution. Do not create a Render Postgres or a second backend instance.
4. Set the backend secrets from `.env.stage1.example`. Generate independent random JWT_SECRET and AI_SERVICE_TOKEN values; never use local test constants. Use PORT provided by Render, leave API_BIND unset, BETA_MODE=true, COOKIE_SECURE=true, BACKGROUND_PROCESSING=false, ENABLE_SANDBOX=false, LLM_ENABLED=false and DB_MAX_CONNECTIONS=4. KAFKA_BROKERS must be absent.
5. **Vercel:** import the existing repository as a Hobby project with root directory `frontend`. Do not enable Git-triggered deployment. Both Vercel configuration files set git.deploymentEnabled=false. Configure API_PROXY_URL to the backend HTTPS URL and pin Node 22. No browser-visible API credential is needed.
6. Set FRONTEND_ORIGIN on Rust to the exact final website origin, with no path/trailing slash. The browser only calls its own `/api`; the server proxy forwards Origin/cookies to Rust. Preview URLs need separately authorized configuration and must not be wildcard-trusted.
7. Only after renewed authorization, manually build/release. The container starts Rust and one internal Python worker. Rust applies the additive migrations to the new testing database. `/live` checks the process; `/ready` checks Postgres. Python's `/ready` is internal loopback. Never expose port 8001 publicly.
8. Create new synthetic `.test` administrator/customer accounts, knowledge and workflows; repeat the full customer journey and tenant-negative checks against HTTPS. Confirm the actual session cookie is HttpOnly, Secure and SameSite=Strict. Verify quota denial and the disabled sandbox.
9. Measure actual Linux memory, cold starts, hosted API latency and function/DB quotas. Stop the service if it exceeds the free memory/quota envelope; do not silently upgrade or add keep-awake traffic.

## Free-host failure behavior

An idle Render instance can sleep, and the database can scale to zero. The website may show a temporary-unavailability/waking message. It must not interpret an HTML provider loading page as a successful API response. Do not automatically replay uncertain writes; inspect the inbox first. The proxy has a bounded timeout and preserves normal API errors.

Workflow actions execute with their ticket/message transaction. Each workflow has a savepoint; failed effects roll back and a failed execution/trace persists. A completed HTTP response means its durable state committed. A connection lost before confirmation remains uncertain and requires inspection. Approval nodes create a human-review request and stop; they do not promise automatic continuation. Durable scheduled delays are deliberately disabled. On-demand notifications recover from committed outbox records without a worker staying awake.

## Rollback and data handling

- Keep automatic deployment disabled. Record the application commit, migration version and provider deployment IDs before any future release.
- For an application rollback, select a previously verified **Stage 1** build that contains every migration already applied to that database. Do not deploy the pre-Stage-1 baseline binary against a database with new migrations; its embedded migrator can reject unknown versions.
- Do not drop tables, reverse migrations, delete volumes or reset managed data as an application rollback. Use a forward fix or a separately rehearsed compatible build. The first Stage 1 deployment has no prior verified hosted release.
- Rehearse restoring a provider snapshot/export into a separate test database. This task did not perform a hosted backup/restore drill. Free retention/history is limited; no production recovery SLA is claimed.
- The public beta accepts synthetic data only. It cannot send external email or charge money. Reset/verification delivery requires a real adapter later; the secret-bearing development sandbox is disabled publicly.

## Release evidence

See STAGE1_RELEASE_REPORT.md, the measured JSON artifacts, screenshots and release branch commit references. Configuration syntax and native tests are evidence of preparation, not evidence that Vercel/Render/Neon were deployed successfully.

## Reproduce final container verification

Start Docker Desktop and select its Linux engine. Install the Python requirements in the existing virtual environment and run npm ci/Playwright Chromium setup in tests as above. From repository root run:

```powershell
backend/ai-copilot/.venv/Scripts/python.exe scripts/verify_containers.py
```

This uses only aster-stage1-verify (web 3000/API 8080) and aster-stage1-free-verify (beta API 18083), with distinct retained named volumes. It creates .env with random local credentials only if absent. Never remove an existing .env or reset volumes to make a test pass. Repeated beta rehearsals intentionally accumulate synthetic organizations and can hit the public organization cap. Stop these two projects with docker compose -p aster-stage1-verify stop and docker compose -f compose.stage1.yaml -p aster-stage1-free-verify stop; no volume deletion is needed. The backend crash, DB outage and recreation tests interrupt only these named rehearsal containers.

Application runtime images have no npm/pip package managers. Update locked dependencies through the build stages and rebuild; do not install packages interactively into running containers. PostgreSQL stays on major version 17. The free-container profile is an HTTP localhost rehearsal with COOKIE_SECURE=false; the future HTTPS deployment must use COOKIE_SECURE=true from the existing Render manifest.
