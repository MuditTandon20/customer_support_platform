# Security verification and limits

This is a local implementation, not a security certification or production-readiness assessment.

Implemented and tested: live membership/role checks; tenant/customer ownership with indistinguishable 404; HttpOnly/SameSite=Strict browser cookies; exact-origin checks on cookie writes; password-reset token hashing, expiry, single use and session-version revocation; no public self-assigned team membership; last-admin guard; local action allowlist and argument checks; source/version validation before cited proposal dispatch; body/IP limits; safe SQL/internal error messages; atomic outbox/audit writes; concurrency/idempotency tests; isolated Python service token; disabled paid inference by default.

Frontend dependencies were upgraded to Next.js 16.4.3 and transitive fixes; `npm audit` reported zero vulnerabilities. Rust was upgraded to SQLx 0.8.6, Argon2 0.6 and patched jsonwebtoken 10.x; compatible dependencies were refreshed. Kafka compression features were removed because the publisher uses NoCompression. The supplied PostgreSQL data and legacy password formats are preserved; a synthetic legacy Argon2 hash is checked in the Rust compatibility tests.

Python uses a separate .venv and requirements.lock constraints. The OSV scan of that interpreter found no reported Python advisories after updates. The user's global Python environment was not altered. Linux-only optional dependencies are not verified by this Windows lock/environment; container execution is still required.

The public OSV dependency inventory is in [dependency-scan.json](dependency-scan.json), with the initial scan retained separately. Remaining Rust transitive findings:

| Dependency | Advisory | Current assessment |
|---|---|---|
| block-buffer 0.10.4 | GHSA-qwgh-2vcv-g2f7 | Older hashing consumers remain alongside patched 0.12.1. No panic-recovery attack path has been demonstrated here; this is not an exemption. |
| crypto-bigint 0.5.5 | RUSTSEC-2026-0387 | Included by general-purpose JWT crypto support. Service issues/verifies HS256 only; other algorithm verification is rejected in tests. Requires continued dependency review. |
| rsa 0.9.10 | RUSTSEC-2023-0071 | Included by JWT crypto support; service does not use RSA signing/decryption. No patched compatible RSA dependency is asserted. Requires continued review. |

Run `backend/ai-copilot/.venv/Scripts/python.exe scripts/dependency_scan.py` to query OSV for Cargo.lock plus installed project Python dependencies. Only public package names and versions are sent. The `active_in_current_target` field distinguishes dependencies selected by the current cargo tree from unused lock entries. A dependency scan does not establish exploitability or completeness; no clean Rust scan is claimed.

Production prerequisites remain: HTTPS and COOKIE_SECURE=true; a same-site API origin; trusted ingress and distributed limits; secret rotation and external secret storage; malware-scanned attachments; verified mail/billing webhooks and provider authorization; complete permission customization; retention/data deletion/export controls; monitoring/alerting; backup/restore drills; production load/security testing. Keep ENABLE_SANDBOX=false in production. Sandbox mailbox access is administrator-only and includes bearer reset/verification links intended for local testing.

Extractive suggestions can reproduce malicious text contained in a knowledge article. Such content is visibly a source excerpt, never an authorization instruction. No model-produced tool call is executed. Human review remains essential; this does not constitute a general prompt-injection guarantee for future generated answers.
