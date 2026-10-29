# Aster Support OS
## The project explained from scratch

A simple-language guide for the project owner, a new teammate or a first-time tester.

Prepared for Mudit Tandon | 29 October 2026

## 1. What is this project?

Aster is a website that helps a team handle customer questions. Think of it as a shared support desk: customers ask for help, agents answer, and the team can see the history instead of searching through personal messages.

It also has an assistant that can help summarize a request and find useful text in the team's knowledge articles. The assistant is there to support the agent. In the current release, it is not a fully autonomous employee that can browse arbitrary sites, issue refunds or promise a correct answer.

The project is also called Customer Service Automation in the repository folder. Aster Support OS is the product name. "OS" here means a connected support workspace; it is not an operating system like Windows.

The current Stage 1 release is a working local product with tested support flows. It is prepared for a small synthetic-data, non-commercial test. It has not been publicly deployed, and the full long-term specification is not finished.

## 2. Why would someone use it?

Imagine a small software company receiving questions such as "How do I reset my password?", "What is your refund policy?" and "Why did my API request fail?" Without a shared system, two agents might answer the same question, one might miss an important reply, and each might quote a different policy.

In Aster, each request has one saved record. The assigned agent can see earlier messages, reference the current policy, write an answer and mark the next step. An administrator can inspect who changed the request and whether the team has a growing backlog.

This can make support more organized. It has not yet been shown to reduce cost, increase revenue or improve resolution rates by a particular amount. Those outcomes require actual user testing and measurement.

## 3. The basic words

| Term | Simple meaning |
|---|---|
| Organization/workspace | One team's private area in Aster |
| Tenant | Technical name for an organization whose data must stay separate |
| Customer | Person asking for help |
| Agent | Team member handling the request |
| Administrator | Person managing the workspace, membership and review decisions |
| Ticket | Saved support request, including subject, status and ownership |
| Conversation/message | The replies associated with a request |
| Internal note | Staff-only message that the customer must not see |
| Knowledge base | Collection of reference articles maintained by the team |
| Citation | Reference showing which article text supported a suggestion |
| Workflow | A saved set of conditions and supported actions |
| Approval | Recorded human-review decision; not proof of an external action |
| Audit log | History of important actions and changes |
| CSAT | Customer satisfaction feedback |
| Synthetic data | Invented test records, not real customer information |

## 4. Who uses which part?

An administrator creates the workspace and invites people. Agents use the inbox and conversation tools to help customers. Customers use the portal to submit requests and see their own replies. The role changes what someone is allowed to do.

An organization is also a data boundary. A request from one company must not appear in another company's inbox or be used to answer its questions. The server checks these boundaries, rather than trusting an organization identifier sent by the browser. A customer also cannot simply change a URL to read another customer's ticket.

The current system uses one organization per user. Advanced custom permission sets and switching one account between several organizations are not available yet.

## 5. A complete example, step by step

Suppose Jamie asks: "How long does a refund review take?" Jamie opens the customer portal, signs in and creates a ticket. Aster saves the request in PostgreSQL and makes it available in that organization's inbox.

An agent opens it, assigns ownership and reviews the conversation. If the reference articles contain a current refund policy, the agent can request a grounded draft. Aster finds permitted article text and provides a suggestion with a source reference. In the demo, the policy says requests are reviewed within five business days; that is fictional demo policy, not a promise about any real company.

The agent reads the suggestion, edits if needed, and sends a public reply. Sending a portal reply means it is stored for the customer to read in Aster. It does not mean an email was delivered. The customer sees the reply and can ask a follow-up question.

When the request is handled, the agent marks the appropriate status. Satisfaction feedback can be collected. The dashboard and audit log reflect actual stored activity. Marking a request solved is an agent decision; it is not independent proof that the customer's underlying problem was resolved.

## 6. What you see in the UI

### Overview

The overview is the entry point to current support activity. It helps you understand the workload and move to the inbox or another module. Its numbers come from records in the current workspace. An empty organization may have little to show, which is why the local demo contains clearly labeled sample records.

### Inbox and conversation

The inbox organizes requests and supports search/filtering. Within a ticket, agents can inspect the conversation, send replies, add private notes, change priority or labels, assign ownership and update the ticket's state. Customers are not meant to see the staff console or private notes.

### Knowledge

The knowledge area stores articles used as reference material. Staff can decide whether an article is for the team or public help. Freshness and archive state matter: an old or withdrawn source should not silently continue supporting an answer.

### Workflows

The workflow studio offers a visual editor for supported nodes and connections, as well as keyboard editing. You can save a draft, simulate it, publish a version and inspect run traces. An editor picture alone is not the workflow engine; the backend validates and executes the supported actions.

### Approvals, analytics and settings

Approvals let administrators inspect requests that need human review. Analytics and AI activity show recorded operations and assistant outcomes. Settings manage the organization. Integration/billing screens must describe their actual state: a visible refund or subscription option does not mean payment processing is connected.

### Customer portal, help center and widget

The portal is the customer's authenticated place to ask and reply. Public help content can be searched separately. The current widget is an authenticated portal launcher, not a complete anonymous live-chat service. Public knowledge must not expose team-only articles.

## 7. How the parts fit together

You do not need to know programming to use Aster, but four parts explain how it works:

| Part | What it does | Analogy |
|---|---|---|
| Next.js frontend | Displays pages and handles user interaction | Reception desk and screens |
| Rust/Axum API | Checks identity/permissions and applies business rules | Responsible office manager |
| PostgreSQL | Stores users, tickets, messages and other durable records | Organized records room |
| Python/FastAPI service | Runs internal local analysis and extractive assistance | Research assistant |

A normal request travels from the browser to the frontend's API proxy, then to Rust. Rust checks permission and reads or writes PostgreSQL. When analysis is needed, Rust calls the protected internal Python service. The response returns through the same chain to the browser.

The frontend cannot grant itself permission. The Python service is not an open public shortcut to customer records. Rust remains responsible for the user's authority and the organization boundary.

## 8. What happens to data?

PostgreSQL is the durable store. The conversation should survive an application restart. Docker containers are replaceable running environments; they are not themselves a backup. The local database uses a retained volume so recreating the application containers does not erase tickets.

Migrations are numbered database changes. A migration might add a table or index required by a new feature. Startup applies missing migrations. Nine were verified for this release, including fresh initialization and retained-data operation. Editing a previously applied migration or deleting a database volume to make a test pass is not an acceptable upgrade strategy.

Some actions also create an outbox event in the same database transaction. This means the data change and the event record succeed together. A later component can process events. It does not guarantee delivery to a real mail or messaging provider. Kafka publishing is optional, and broker runtime/consumer recovery is not part of the completed Stage 1 proof.

During today's maintenance, retained rehearsal databases were backed up before index/collation maintenance. Checks found unchanged row fingerprints across their public tables afterward. A backup was taken, but a separate restore drill has not been completed; those are different claims.

## 9. What the AI does today

The default assistant does not require an OpenAI key or paid model. It uses deterministic rules and extracts relevant text from stored knowledge. An article is split into overlapping chunks so retrieval can return a useful part rather than an entire long document. The Stage 1 implementation uses 180-word chunks with 30-word overlap and PostgreSQL text search.

This is lexical retrieval: it matches indexed text. It is not semantic vector search that understands every reworded question. Suggestions include references so an agent can inspect their basis. When evidence is missing, abstaining is safer than inventing policy.

Before dispatching a saved proposal, the server rechecks the supporting sources. If an article becomes stale, inaccessible or changes version, the old proposal should not simply be sent as if nothing changed. Staff remain responsible for the final response.

The assistant can also summarize and apply simple intent rules. These rules are not calibrated confidence estimates. Tone preferences can be stored without implying that the system rewrites quoted policy in that tone. No multilingual generative-quality guarantee is made.

In public-beta mode, paid inference remains disabled even if a key is present. A Python outage does not remove the agent's ability to write a manual answer. AI suggestions are a convenience, not the only way to use the system.

## 10. What automation does and does not do

Supported ticket events can run a published workflow. For example, a workflow may apply a label, set a supported internal field, create a notification or request human review. Execution is bounded to avoid uncontrolled loops. Versions and traces make it possible to see which saved definition ran and what happened.

Failure handling matters. If an action fails, the supported transactional changes are rolled back while a failure record is retained. Repeating an action with the same valid idempotency key must not silently duplicate its effect. An idempotency key is a request identifier used to recognize that repeat.

Sensitive external tools remain blocked when their provider adapter does not exist. Recording approval for a cancellation does not cancel a real subscription. Human-review checkpoints exist, but automatically resuming the entire workflow afterward, durable waiting timers and a general retry engine are future work.

## 11. Security explained without jargon

The first rule is that the right person must see only the right records. Login, live roles, organization checks and customer ownership checks support that rule. The browser uses an HttpOnly session cookie, which normal page scripts cannot directly read. Origin checks help protect cookie-authenticated actions. Rate and request-size limits bound abuse.

The local containers run with limited privileges. They use non-root accounts, and the tested Compose configuration removes effective capabilities, disallows privilege gain and makes the main filesystem read-only. Small temporary/cache locations remain writable where needed. Programs with special SUID/SGID permissions were stripped from runtime images.

Dependency scanners still report vulnerable packages. The PostgreSQL image, for example, retains critical package matches. The review found that the affected XML operations are not exposed by the current JSON/text application paths, and the updated gosu helper contains zero detected affected linked symbols. This does not mean every library is patched or every possible attack is impossible.

The assessment applies to the exact reviewed feature set and controls. New uploads, XML parsing, arbitrary SQL, shell tools, dependency changes or real-customer use require fresh review. The release is suitable for its narrow synthetic test scope, not a production security certification.

## 12. How to open the dashboard now

Use the same computer where Docker Desktop and this project are running. Open http://localhost:3000/login and sign in. The dashboard is http://localhost:3000/workspace/dashboard. The shorter /workspace path is not the dashboard route.

A new, separate sample workspace was created for this documentation handoff. It contains three invented customers, six sample tickets, three reference articles, a review workflow and a blocked sensitive-action example. All fixtures are labeled Demo. Existing organizations were not reset or overwritten.

Local demo email: demo-admin-09e4ef@example.test

Local demo password: Demo-only-password-2026!

These are intentionally shareable local synthetic-demo credentials, not production credentials. Never reuse this password for a real account or deploy this demo account with real customer data.

After signing in, open Overview, then Inbox. Select a Demo ticket and inspect its messages, priority and labels. Open Knowledge to see the sample reference articles. Inspect Workflows and Approvals to understand their saved state. You can try the theme toggle and a narrow/mobile window. A blocked cancellation record is expected; no real payment account exists.

The authenticated portal starts at http://localhost:3000/portal. Public help uses article-specific links; follow the product's help links for a selected public article rather than assuming a generic /help page exists.

Localhost means this computer. It is not a public internet link and will not open this machine's project on your phone merely by clicking it there. No hosted dashboard URL exists yet.

## 13. Starting or checking the local project

If the current stack is already running, you do not need to rebuild it just to look at the UI. If it has been stopped, open Docker Desktop with its Linux engine and use PowerShell in the project folder. For the retained verified preview, run docker compose -p aster-stage1-verify up -d --wait. Use that exact project name to reconnect the existing rehearsal volume.

For a separate first-time setup, the README describes scripts/init-env.ps1 and docker compose up --build. A different Compose project name can create a different database volume, so an empty workspace there does not mean the retained data disappeared. Never remove volumes as a troubleshooting shortcut.

The default local endpoints are frontend http://localhost:3000 and API readiness http://localhost:8080/ready. Internal AI and database ports are not published by the hardened local Compose profile. The separate constrained beta API rehearsal uses port 18083; it is not a second dashboard.

If a page says Failed to fetch, check that the Rust API is running, /ready is healthy and the frontend proxy points to the intended API. If signup reports an existing record, sign in or use a different test email; do not erase the database. If an automated test receives 429, it must respect the rate window rather than weakening the app's limit.

## 14. What was tested today?

The reviewed application commit is d83daf1c50dc204493889560156175d081cc5798 on release/stage1-testing. GitHub Actions workflow #9 passed both integrated and container jobs. The result is available at https://github.com/MuditTandon20/customer_support_platform/actions/runs/40900733408.

Tests passed: five Rust unit tests, one isolated Rust advisory regression, 22 Python tests, 18 full-stack API suites, seven constrained beta suites and eight browser tests. Frontend lint, types, build and npm audit passed. Container checks exercised all nine migrations, readiness, service/database failures, stop/start, recreation and supervised worker recovery.

The security review also checked actual container privileges, absence of special-permission binaries, JSON/XML boundaries, literal storage of XML-looking text, forbidden shell-tool rejection and source/config scope. New unknown findings or changed reviewed sources are designed to fail the assessment gate.

Earlier CI failures are still visible in GitHub history. Runs #7 and #8 hit an authentication throttle because earlier abuse tests had consumed the allowance. The verifier was fixed to retry only 429 for a bounded period. This changed test handling, not the production rate limit. The later green run verifies the corrected commit.

## 15. What is still unfinished?

Live email, WhatsApp, real payments, production provider webhooks, attachment storage/scanning, semantic embeddings, durable workflow timers/resume, advanced custom permissions and multi-organization membership are not complete. Production telemetry, retention/deletion policy, backup restoration, sustained-load testing and hosted behavior need further work or verification.

Do not interpret an integration settings screen as a connected provider. Do not interpret a synthetic workflow run as proof of a real-world refund. Do not interpret a solved counter as measured customer success. Keeping these distinctions visible is part of the product's design.

## 16. What does the zero-budget plan mean?

The prepared plan uses a free frontend host, one small free backend instance and a free managed PostgreSQL account. The selected project documents name Vercel Hobby, Render Free and Neon Free. The Rust and Python processes share a single constrained backend container in that layout.

The ₹0/month target depends on eligibility, non-commercial use and staying within actual free quotas. A free service may sleep, cold-start slowly or stop serving when a quota is exhausted. Pricing and rules must be rechecked before a future launch. No cloud accounts, paid resources or public deployments were created as part of today's work.

A future authorized launch must verify HTTPS, secure cookies, secrets, exact allowed origin, database access controls, runtime privileges and beta restrictions on the actual host. Local Docker success does not prove a provider applies every Compose security flag. If required controls cannot be met for free, use the local rehearsal instead.

## 17. Where to look in the repository

| Location | What you will find |
|---|---|
| frontend/ | The Next.js pages, components and frontend API proxy |
| backend/rust-api/ | Identity, business rules, API handlers and database migrations |
| backend/ai-copilot/ | Internal Python analysis and extraction service |
| compose.yaml | Full local service configuration |
| compose.stage1.yaml and deploy/ | Constrained beta rehearsal and image definitions |
| tests/ | Unit, API, browser, recovery and security tests |
| scripts/ | Local setup, demo seeding and verification helpers |
| docs/ | Product design, architecture, reports, evidence and these documents |
| .github/workflows/ci.yml | Automated verification; not an automatic deployment pipeline |

The PRD explains what the product should do within Stage 1. This guide explains how to understand and try it. The daily task record saves what was done and what the next person must preserve. For exact security evidence, use STAGE1_SECURITY_CLOSURE.md and the associated JSON reports rather than relying on a summary alone.

## 18. Current handoff

The local application and latest reviewed CI are working. You can explore the dashboard with synthetic records. The release is eligible for a controlled, non-commercial Stage 1 test of the reviewed build. It is not publicly deployed, not fully production-ready, and not a completed version of every feature in the original product vision.
