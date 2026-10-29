# Stage 1 security review closure

2026-10-29. **Eligible for a synthetic, non-commercial Stage 1 test deployment of the reviewed build. Not deployed.** The earlier security hold is resolved within this scope, subject to the final integrated/container CI gates being green. This is an application-specific assessment, not a clean-package scan, independent audit or production security certification.

## Fixed and hardened

- Updated the PostgreSQL image's `gosu` helper from 1.19 to the official 1.20 release using SHA-256-pinned amd64/arm64 assets. PostgreSQL stays at major version 17; no application migration was rewritten. Only amd64 runtime execution was tested.
- Analyzed the exact old and new helper binaries with **govulncheck 1.9.0**. The old binary contains three affected symbols for GO-2026-6686 in its user-parsing dependency; the new binary contains **zero detected affected symbols**. Go version/package matches remain and are retained. Binary analysis is a symbol inventory, not a complete source-level call graph. See [before/after evidence](STAGE1_GOSU_BEFORE_AFTER.json) and [current binary analysis](STAGE1_GOSU_ANALYSIS.json).
- Removed curl and its library from the Rust API runtime. The API executable now provides a bounded, loopback-only `--healthcheck` mode using its existing HTTP library. Healthy service, dead port, missing credentials and database-outage cases were exercised. This removes four high package matches associated with two curl CVEs.
- Removed SUID/SGID permission bits from all five runtime images without removing their package inventories. Verified non-root processes, zero effective capabilities, no-new-privileges, read-only roots and absence of homed/TPM daemon/device paths. Writable temporary/cache locations are isolated tmpfs mounts. The Next cache is writable by its actual non-root UID.
- PostgreSQL now starts as its own unprivileged OS user. Fresh initialization, all nine migrations, existing-volume startup, support flows, backup/index maintenance and restart/recreation work with the new runtime controls.
- Added source/import boundary checks, JSON/XML rejection and literal-text persistence checks, unsupported-shell-tool rejection, an isolated Rust advisory reproduction, raw image scans and a fail-closed security assessment to CI. No scanner ignore rules or blanket CVE suppression were introduced.

The upstream [gosu 1.20 release](https://github.com/tianon/gosu/releases/tag/1.20) and [security policy](https://github.com/tianon/gosu/blob/master/SECURITY.md) explain the dependency update and version-only scan limitations. The analysis uses the [official Go vulnerability tooling](https://go.dev/doc/tutorial/govulncheck).

## Findings retained and assessed

Raw package-match counts remain visible in [STAGE1_IMAGE_SECURITY.json](STAGE1_IMAGE_SECURITY.json):

| Runtime image | High | Critical |
|---|---:|---:|
| Rust API | 43 | 0 |
| Python AI | 44 | 0 |
| Next.js | 43 | 0 |
| Combined beta backend | 44 | 0 |
| Local PostgreSQL | 86 | 2 |

These are package matches, with duplicates across binary packages, not unique exploitable CVE counts. The complete per-image/component dispositions and sources are in [STAGE1_SECURITY_REVIEW.json](STAGE1_SECURITY_REVIEW.json). It records **zero unreviewed findings** for the current reviewed images and source/config/dependency digest.

| Finding family | Application-specific conclusion and evidence |
|---|---|
| PostgreSQL libxml2, including critical CVE-2026-6653 | The library remains vulnerable, but its affected XML operations are not invoked by the current public JSON/TEXT API. Raw XML requests are rejected; XML-looking ticket/article content remains literal text without entity expansion. Storage types are independently checked as TEXT, and current Rust SQL/Python imports expose no XML parser or arbitrary SQL interface. This conclusion does not cover trusted operators executing XML SQL directly or a future XML/upload feature. |
| Go standard-library matches in gosu, including critical CVE-2025-68121 | The exact updated binary has no detected affected linked symbols. Each retained match is linked to a scoped Go advisory and the binary scan; unscoped/unknown findings fail the gate. The utility is not a web/TLS server, and the tested PostgreSQL process starts directly as its non-root user. |
| util-linux mount/nsenter CVEs | Their privileged operator, SUID mount and namespace/cgroup conditions are absent from application execution. Images have no SUID/SGID programs, processes have no effective capabilities, and current APIs do not execute shell commands. The rehearsal additionally enforces no-new-privileges/read-only roots. |
| systemd-homed and GnuPG TPM CVEs | The affected daemons and TPM devices are absent from the tested runtime images. Neither feature is part of the application. |
| libacl, ncurses infocmp and Perl Archive::Tar CVEs | Current APIs do not invoke privileged path/ACL operations, infocmp on user terminfo, or Perl archive ingestion. User-controlled inputs remain bounded JSON/text; arbitrary process tools and executable/file/archive uploads are absent. Non-root/no-SUID controls provide additional protection. |
| Rust block-buffer 0.10.4 advisory match | The upstream affected range starts at 0.11.0. Version 0.10.4 uses a separate position field and passes an isolated reproduction of the upstream caught-panic scenario. Raw OSV output remains unchanged; no semver-incompatible dependency substitution or migration-checksum change was made. This is a version/source-specific adjudication, not a general memory-safety proof. |

PostgreSQL full-text indexing uses its own built-in token parser, not the SQL XML/DTD operations at issue. Its [parser documentation](https://www.postgresql.org/docs/17/textsearch-parsers.html) and [implementation](https://github.com/postgres/postgres/blob/REL_17_STABLE/src/backend/tsearch/wparser_def.c) support that distinction. The [Debian tracker](https://security-tracker.debian.org/tracker/CVE-2026-6653) still marks the XML library issue in stable; no unstable library was mixed into the image. The Rust [upstream advisory](https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7) and [reproduction result](STAGE1_BLOCK_BUFFER_REPRODUCTION.json) support the narrower Rust finding disposition. Miri was not run.

## Verification and data preservation

- Five Rust API unit tests and the additional isolated advisory regression pass. Twenty-two Python/configuration tests pass, including four fail-closed assessment tests.
- All 18 full-stack API suites, all eight browser/component/mobile/accessibility tests and seven constrained beta API suites pass against hardened containers. All nine additive migrations work on a freshly initialized non-root PostgreSQL service.
- Service/database outages, manual support during AI failure, full stop/start, container recreation, rate limits and supervisor crash recovery pass. The combined backend remains constrained to 0.1 CPU/512 MiB. These are small synthetic tests, not a capacity/SLA guarantee.
- [Runtime security evidence](STAGE1_RUNTIME_SECURITY.json) records actual image IDs, UIDs, capability/privilege/filesystem states, cache access, parser/process boundaries and source digest.
- Fresh security rehearsals use separate projects/volumes on ports 13001/18084/18085. The earlier stacks were then upgraded in place; [before/after counts](STAGE1_EXISTING_DATA_UPGRADE.json) are unchanged, including 43 users/66 tickets in the full rehearsal and 9 users/30 tickets in the beta rehearsal. The usual local website on port 3000 remains available.
- Retained databases had collation metadata from their earlier Debian base. Private backups were taken, affected user/system indexes rebuilt before version refresh, and row fingerprints across all 22 public tables in each cluster remained identical. No data reset, schema rewrite, migration-checksum change or existing-volume deletion occurred. [Repair evidence](STAGE1_COLLATION_REPAIR.json) records the checks. PostgreSQL intentionally leaves template0's default collation version NULL; that was preserved and template0 connections are disabled.

The collation repair was operator maintenance on these owned local rehearsal clusters, not an automatic application migration. PostgreSQL documents [index rebuilding](https://www.postgresql.org/docs/17/sql-reindex.html) and [refreshing collation metadata only after rebuilding affected objects](https://www.postgresql.org/docs/17/sql-altercollation.html). Early maintenance attempts were corrected to batch catalog refreshes and retain template0's intentional NULL version; final checks report no tracked database/named-collation mismatch or invalid index.

## Reproduce and keep the assessment valid

From an installed project environment with Docker's Linux engine running:

```text
python scripts/check_block_buffer.py
python scripts/verify_containers.py
python scripts/review_security.py
```

The last command preserves raw Trivy/govulncheck JSON in `.security-evidence/` and generates the reviewed reports. It fails on unknown image findings, detected affected Go symbols, missing/stale runtime evidence or a change to the reviewed source/config/dependency digest. The committed `STAGE1_REVIEW_SCOPE.json` is updated only after an explicit fresh engineering review, not automatically by CI. The raw results are uploaded by the container CI job. Private database backups are outside the repository and are not uploaded.

Use a fresh rehearsal database for repeated public-beta tests; do not reset retained user data or raise the public quota to make tests pass. The verifier supports the two isolated `aster-stage1` and `aster-stage1-security` prefixes, with configurable localhost ports. Repeated beta fixture creation intentionally reaches the organization limit.

Reassess before changing source/images/dependencies, adding XML/files/archives/URL ingestion, exposing shell tools or direct SQL, relaxing non-root/no-SUID protections, collecting real customer data or making commercial/production claims. Continue taking supported upstream fixes when available. Scanner alerts have been assessed for this build, not permanently waived.

## Future launch gates

Automatic deployment remains disabled. Do not deploy without later authorization. The existing Vercel Hobby + one Render Free combined backend + Neon Free plan still targets **₹0/month only within eligible free quotas**; no accounts, paid plans or resources were created.

The image-enforced non-root/no-SUID design and reviewed feature boundaries are required. Compose additionally enforces capability removal, no-new-privileges and read-only roots; do not claim a hosting platform applies those Docker flags without checking it. At the future authorized launch, verify the actual UID/effective privileges, HTTPS/Secure cookies, exact origin, secrets, database access controls, beta flags, disabled sandbox/model billing and the complete support flow. If required controls cannot be met for free, stop and use the local rehearsal. Hosted cold starts, provider behavior, backup restore drills and field INP remain unverified; no production readiness or uptime promise is made.
