# Final deployment-readiness verification

2026-10-29. **Public test release: SECURITY HOLD — do not deploy.** Container functionality has been exercised; this report supersedes the earlier conditional readiness wording. No cloud deployment, paid resource, merge into main, force push, database reset or volume deletion was performed.

## What changed

Docker Desktop was stopped. It was started with its existing Linux/WSL 2 configuration; Docker Engine 29.7.2 and Compose 5.4.0 became available. Existing projects/volumes were inspected and left intact. All Docker mutations were confined to two named synthetic rehearsal projects, `aster-stage1-verify` and `aster-stage1-free-verify`, and their own named PostgreSQL volumes. The Windows fault/restart/beta profile was also rerun with the rebuilt native API against fresh uniquely named databases in the earlier disposable Postgres 18 cluster; existing records were not reset.

The four-service layout runs production standalone Next.js, optimized Rust, internal Python 3.12 and PostgreSQL 17. The second layout exercises the actual `deploy/Dockerfile.stage1` combined Rust/Python supervisor at **0.1 CPU and 512 MiB**, with a separate persistent Postgres service. Neither layout is a public deployment. The database major version remains 17; migrations and application records are retained across container recreation.

- Added explicit Rust/Next readiness health checks and healthy dependency ordering, with polling disabled and configurable localhost ports.
- Removed unused RSA/ECC JWT providers on **both Windows and Linux**. A small adapter implements jsonwebtoken's documented provider interface using `ring` HMAC-SHA256. JWT parsing, claims/expiry and validation remain in jsonwebtoken. No hand-written cryptographic primitive or replacement token parser was introduced.
- Added an independent legacy HS256 fixture plus signature tampering, wrong-key, expiry and algorithm-rejection checks. Existing Argon2 hashes remain compatible.
- Updated runtime images to Debian Trixie and applied stable distribution updates during builds. Removed npm/yarn/corepack from the standalone Node runtime and pip/ensurepip from Python runtimes after installing locked application dependencies. Build-time pip is updated first. This removes unused package-manager attack surface and their embedded vulnerable dependencies; it does not conceal or suppress remaining OS findings.
- Deferred the external model SDK import until explicit model opt-in. Extractive beta startup does not load it, even with an accidental key. A subprocess regression test checks this behavior.
- Rust release unit tests and frontend lint/typecheck/build/audit execute during image builds. Added a reproducible container verification script and an independent GitHub container job. The workflow contains no deployment step.
- API tests now honor the configured frontend origin. The workflow browser test waits for its actual publish response before checking the completion message; one prior run exceeded the five-second UI assertion while PATCH/publish both returned 200. It does not retry or replay writes.

## Executed checks and evidence

| Check | Evidence |
|---|---|
| Rust native and Linux release unit tests | 5 passed, including legacy password/token compatibility |
| Python native and actual runtime-image unit/config/contract tests | 18 passed, including Linux-only installed extras and lazy SDK startup |
| API integration | 11 existing plus 7 Stage 1 suites against the complete Compose stack |
| Browser/component/security/mobile/accessibility | 8 tests against the production container website, including real cited customer replies, CSAT, XSS text rendering, fixed-host proxy paths and workflow drag/keyboard editing |
| Migrations | All 9 expected migration versions present and successful in fresh Postgres 17; no migration file or existing checksum changed |
| Outages/restarts | AI outage preserves manual replies; database outage makes readiness return 503 and recovers; service restart, complete stop/start and forced container recreation preserve ticket data |
| Windows native fault/beta profile | All 7 AI-outage suites, persisted-ticket restart and concurrent organization cap checks passed with the new HS256 provider; earlier native preview restored with readiness 200 |
| Local security controls | Non-root application processes; Python has no published host port; missing AI service token denied; hostile Origin denied even with bearer auth; SQL-like search treated as data; 31st authentication request rate-limited |
| Constrained combined backend | Stage 1 API suites pass with beta guards enabled under 512 MiB/0.1 CPU; AI-worker termination triggers supervisor/container recovery and retained ticket remains accessible |
| Dependency checks | npm project/test audits clean; installed Python application OSV clean; one retained Rust advisory-data discrepancy below |
| Docker image scan | Trivy 0.76.1 HIGH/CRITICAL scan completed, findings retained in STAGE1_IMAGE_SECURITY.json |

The final reproducible command is `backend/ai-copilot/.venv/Scripts/python.exe scripts/verify_containers.py` on Windows, or the equivalent virtual-environment interpreter on Linux. The final local run exited **0** and printed ?Container functional verification passed.? CI references are reported at handoff. `STAGE1_CONTAINER_CHECKS.json` and `STAGE1_FREE_CONTAINER_CHECKS.json` record actual recovery results. Images and synthetic volumes are deliberately retained.

Earlier failures remain relevant: the first image build hit a Python download timeout and was rebuilt with bounded download retries; the first constrained signup attempt timed out without a committed organization, followed by successful timed signup and full API suites. No uncertain write was automatically replayed. The first final browser pass had a publication assertion timing failure; the response-aware test described above fixes the test's completion boundary. These are not an availability guarantee.

## Three original Rust findings

| Finding | Investigation and disposition |
|---|---|
| rsa 0.9.10 / RUSTSEC-2023-0071 | **Removed from Cargo.lock and both target dependency graphs.** No compatible patched RSA release is asserted. Only HS256 is supported. [RustSec](https://rustsec.org/advisories/RUSTSEC-2023-0071.html) |
| crypto-bigint 0.5.5 / RUSTSEC-2026-0387 | **Removed with the unused asymmetric provider graph**, rather than forcing a semver-incompatible transitive update. [RustSec](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) |
| block-buffer 0.10.4 / GHSA-qwgh-2vcv-g2f7 | **Raw OSV finding retained, advisory-data discrepancy unresolved.** SQLx still uses SHA-2 0.10 for migration checksums. The [upstream advisory](https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7) currently lists affected versions as >=0.11.0,<0.12.1; OSV currently reports all versions before 0.12.1. Inspected 0.10.4 stores its position separately, unlike the inline-buffer implementation in the advisory. This supports a likely metadata false positive, not a blanket safety exemption. No unsafe version override, checksum change or scanner suppression was added. |

Current Windows and Linux/container-Python inventories are `dependency-scan.json` and `dependency-scan-linux.json`. The scanner now supports target triples and actual image inventories. Only public dependency names/versions are sent to OSV.

## Image findings and remaining security gate

Final HIGH/CRITICAL package-finding counts (duplicates across installed binary packages are not unique CVE totals):

| Image | High | Critical | Scanner lists available fixed version |
|---|---:|---:|---:|
| Rust API | 47 | 0 | 0 |
| Python AI | 44 | 0 | 0 |
| Next.js runtime | 43 | 0 | 0 |
| Combined beta backend | 44 | 0 | 0 |
| Local PostgreSQL 17/Trixie | 86 | 2 | 26 |

The two local-DB critical matches are **CVE-2026-6653** in libxml2 and **CVE-2025-68121** in the Go standard-library build metadata of the upstream `gosu` utility. Aster is still Rust/Python/Next; this utility comes from the official Postgres image.

- Debian marks the libxml2 issue as vulnerable in stable Trixie with no DSA (minor issue); a fixed version exists in unstable. Do not mix unstable libraries into the verified database image as an untested fix. Aster does not expose arbitrary SQL/XML parsing or publish the DB port, reducing reachable attack surface; this is not a claim that the library is patched. [Debian tracker](https://security-tracker.debian.org/tracker/CVE-2026-6653).
- The Go finding concerns TLS session-resumption behavior. `gosu` switches users and executes a process. Its [maintainer security policy](https://github.com/tianon/gosu/blob/master/SECURITY.md) explains why version-only Go scans can flag unused functionality and recommends `govulncheck`; the current [1.20 release](https://github.com/tianon/gosu/releases/tag/1.20) still uses Go 1.24.6. **Reachability-aware govulncheck was not run** because no Go/govulncheck toolchain is installed here. The raw finding remains, along with its related version-only high matches. [Go advisory](https://pkg.go.dev/vuln/GO-2026-4337).
- Docker Scout required account login, so it was not used and no account/payment method was created. Trivy supplied the actual completed image scans instead.

The local database image is not the selected managed Neon deployment, but these findings must not be silently ignored. There is **no clean-image claim and no blanket vulnerability acceptance**. Complete a recorded reachability/risk review or move to a patched supported image/provider before public release. Do not “fix” scanner output by suppressing findings, removing package inventories, weakening password hashing, rewriting migrations, or mixing unsupported libraries.

## Performance and unverified scope

The final recovery run measured service restart **3438 ms**, database recovery **8781 ms**, complete stack start **36938 ms**, and complete container recreation **39390 ms**. These include the local Compose command and health-gate waits, not provider cold starts. The constrained backend passed all seven Stage 1 API tests in **18.087 s**; its AI-worker crash recovery measured **4437 ms**. Cgroup peak before that crash was **73,986,048 bytes (70.6 MiB)**; the restarted cgroup peak was **35,418,112 bytes**, with no OOM kill. These snapshots cover this small rehearsal, not a sustained-capacity guarantee.

The container API smoke check made **100 requests**, concurrency **10**, **0 errors**, p50 **14.36 ms**, p95 **271.85 ms**, **254.8 requests/s**, in an empty synthetic tenant. This is a smoke measurement, not an SLO.

See `STAGE1_CONTAINER_LIGHTHOUSE.json`, `STAGE1_CONTAINER_INTERACTIONS.json`, `STAGE1_CONTAINER_LOAD.json` and the two recovery JSON reports. Measurements are local Docker/Windows browser observations, not Render uptime/cold-start measurements or field Web Vitals. A contended Lighthouse run measured 70 performance/100 accessibility, LCP 3079 ms, CLS 0 and TBT 1184.5 ms, with a cache-clear-timeout warning; the original native 81 score is not a container score. That run is preserved as STAGE1_CONTAINER_LIGHTHOUSE_CONTENDED.json. A second run after functional tests measured **74 performance/100 accessibility**, **3106.69 ms LCP**, **CLS 0**, **834.5 ms TBT**, with no warnings/runtime error. Neither result is a production target or proof of a stable regression/improvement. The second interaction sample recorded 32 ms Event Timing entries and an accumulated layout-shift value of 0.2344 across auth-page navigation; this raw across-navigation sum is not field Core Web Vitals CLS. Field p75 INP and hosted HTTPS behavior remain unverified. Do not treat synthetic Event Timing samples as field INP.

No public endpoint was deployed, no managed database/account was provisioned, and no hosted backup/restore, cold-start or provider-quota test was performed. The ₹0 plan remains conditional on the documented free quotas/eligibility. Follow STAGE1_DEPLOYMENT_RUNBOOK.md only after the security hold is resolved and a later deployment is authorized.

## Visual evidence and Git scope

[Initial CI run 40882403937](https://github.com/MuditTandon20/customer_support_platform/actions/runs/40882403937) passed the integrated job and all container API/browser tests, then exposed an older-Compose incompatibility in the harness: `compose start --wait` was unsupported. The harness now uses `start` plus explicit inspected health polling. The complete local recovery sequence passed again with that portable implementation; the follow-up CI conclusion is reported at handoff. The native profile script also no longer writes an obsolete Docker-unavailable conclusion for a check it does not perform.

Actual final container screenshots: [desktop](screenshots/workspace-desktop.png), [mobile](screenshots/workspace-mobile.png), [light theme](screenshots/workspace-light.png), [customer portal](screenshots/customer-portal.png), [workflow studio](screenshots/stage1-workflow.png). Only release/stage1-testing is committed/pushed. Main stays at d3b4e25; prior release baseline is ba39a46. The original PDF, HTML and PDF generator remain untracked and untouched. No Kubernetes or replacement application stack was introduced.
