> Current Stage 1 results supersede historical counts: see [STAGE1_RELEASE_REPORT.md](STAGE1_RELEASE_REPORT.md). Docker/cloud runtime remains unverified.

# Verification report — 2026-10-29

The connected local implementation is demonstrated. The complete supplied PRD is not finished, and this report does not claim production readiness. Exact capability status is in [requirements-traceability.md](requirements-traceability.md).

## Final checks

| Command/check | Result |
|---|---|
| `cargo check --manifest-path backend/rust-api/Cargo.toml` | PASS after dependency upgrade |
| `cargo test --manifest-path backend/rust-api/Cargo.toml --locked` | PASS, 4 tests: terminal-state policy, graph safety, legacy/current Argon2 compatibility, JWT algorithm restriction |
| `backend/ai-copilot/.venv/Scripts/python.exe -m unittest discover -s tests -p 'test_*.py' -v` | PASS, 13 tests: retrieval/evidence/escalation fixtures, contract inventory, service auth/validation and disabled ungrounded generation |
| `backend/ai-copilot/.venv/Scripts/python.exe tests/integration.py` | PASS, 11 suites against the rebuilt stack: tenant/customer boundaries, assignment, notes, idempotency, source freshness on send, tools/approval, workflow version restore/simulation/execution, metrics, CSAT, sandbox round trip, notifications, concurrent requests, cookies, verification/reset |
| `node fresh-stack.mjs` from tests | PASS: fresh empty database, all migrations, same 11 integration suites; [artifact](fresh-migration-verification.json) |
| `npm run lint` from frontend | PASS, no reported warnings/errors |
| `npx tsc --noEmit` from frontend | PASS |
| `npm run build` from frontend | PASS on Next.js 16.4.3, including all 16 route entries in the generated route report |
| `npm audit` from frontend | PASS, zero reported vulnerabilities after framework/transitive updates |
| `npm run test:e2e` from tests | PASS, 3 Playwright tests: agent flow; customer portal/CSAT/widget; marketing/login/reduced motion |
| Playwright + axe | No serious/critical findings in the tested WCAG 2 A/AA and 2.1 AA tags: dashboard, login, customer conversation, mobile inbox/knowledge/workflows/customers/approvals/integrations/billing/settings/audit, and light dashboard. This is not full WCAG 2.2 AA certification. |
| `docker compose config --quiet` and events override config | PASS, including optional broker/publisher override |
| Docker container builds/runtime | NOT RUN: Docker Desktop Linux engine unavailable. Native stack ran successfully. |
| Hosted CI workflow | AUTHORED, NOT RUN: no remote push/hosted job |
| OSV Cargo/Python scan | Completed, 340 dependencies inspected. Isolated Python has no reported findings; three Rust transitive findings remain. [Security assessment](security.md), [scan data](dependency-scan.json) |
| `git diff --check` | PASS; Git reports ordinary LF/CRLF conversion notices, no whitespace errors |

## Local smoke measurement

`backend/ai-copilot/.venv/Scripts/python.exe tests/load_smoke.py` completed 100 requests with concurrency 10 and zero errors against the latest Rust debug build and disposable PostgreSQL 18. Actual recorded results: p50 52.2 ms, p95 295.41 ms, 0.98 seconds elapsed. Half the requests read inbox, half analytics, in an empty synthetic tenant. See [load-smoke.json](load-smoke.json). This is a basic concurrency smoke check, not a production load test, capacity guarantee, or availability SLO.

## Visual evidence

Screenshots were captured from the running application, not generated mockups. Demo fixture records and workspace names are explicitly labeled.

- [Demo dashboard](screenshots/demo-dashboard.png)
- [Demo inbox](screenshots/demo-inbox.png)
- [Demo workflows](screenshots/demo-workflows.png)
- [Demo mobile](screenshots/demo-mobile.png)
- [Customer portal](screenshots/customer-portal.png)
- [Light theme](screenshots/workspace-light.png)
- [Mobile sign-in](screenshots/login-mobile.png)

## Limits and unverified requirements

Live email/WhatsApp/payments, semantic embeddings/ingestion queues, attachment handling/scanning, custom permissions/multiple memberships, complete durable workflow execution, independently verified resolution, full external integration/developer platform, retention/backups, production observability and production security remain incomplete. Provider credentials alone do not supply missing adapters. No paid inference/provider call, email delivery, payment, deployment or production data change was performed. Stage 1 authorizes a separate testing-branch push.

Some early checks failed and were repaired: browser selector matched the workflow icon incorrectly; login inline links needed underlines; animated text needed settling before contrast measurement; the mobile audit table needed keyboard focus. Final browser checks passed after these corrections. The initial dependency audit found inherited vulnerabilities, leading to the upgrades and explicit remaining-risk report above.

## Sign-in connection repair

Conflicting frontend API ports could leave a page calling an unavailable service. Browser API calls now use the website's same-origin /api route. API_PROXY_URL is server-only runtime configuration, set to port 18080 for the current preview and api:8080 in Compose. Cookies, authorization, Origin and upstream status codes are preserved; requests remain bounded at 256 KiB. Unavailable upstream services return a readable 502 response.

After this repair: frontend lint/typecheck/production build pass; two sign-in/signup regression tests and the three existing browser flows pass. Proxy checks return health=200, missing authentication=401, oversized request=413. The actual supplied demo account was verified in a browser: login, session, analytics and inbox requests all returned 200 and the dashboard rendered.
