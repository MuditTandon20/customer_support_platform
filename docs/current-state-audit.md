# Baseline audit — 2026-10-29

Source: existing repository and the supplied AI_Support_Full_Build_Package.zip, read in full. The package is implementation input; claims in it are not evidence of working functionality.

- Preserve existing uncommitted Rust dependency/user/model changes and frontend ticket components.
- Rust/Axum, SQLx/Postgres, Argon2/JWT, ticket endpoints, outbox publisher, Next 16.3.3/React 19 and FastAPI exist.
- `cargo check --locked` in backend/rust-api passed. `npm run lint` passed. Frontend production build pending at baseline.
- No consumer/offset implementation, tests, full local stack, knowledge, messages, approvals, workflow execution or billing implementation found.
- Security gaps: public organization join allowed self-assigned agent; extractor trusted stale role claims; unrestricted status transitions; cross-tenant assignment unchecked; SQL errors exposed; permissive CORS; no request timeouts. Existing ticket read ownership correctly returns 404.
- Docker CLI installed, Linux daemon unavailable at baseline. No listening PostgreSQL on localhost:5432. Existing SQLx build metadata/environment does permit compilation; do not expose local secrets or connect to an unknown configured database for destructive tests.

Validation and remaining requirements are recorded in requirements-traceability.md and BUILD_STATE.md.
