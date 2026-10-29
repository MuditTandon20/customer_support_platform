# Stage 1 hosting decision — checked 2026-10-29

**Prepare Vercel Hobby + one Render Free combined backend + Neon Free. Do not deploy yet.** Target cost is ₹0/month, conditional on account eligibility and staying inside each free quota. No resources/accounts/payment methods were created. No paid fallback or unlimited availability is promised.

## Selected topology

Next.js stays on Vercel. Its same-origin server proxy forwards authenticated requests to Rust on Render. A single container supervises the existing Rust and Python processes; Python listens only on loopback. Both share one free web instance rather than consuming two overlapping quotas. Managed Postgres stores conversations, chunks, proposals, workflows/runs and receipts. No persistent container disk, Redis, broker or paid inference is required for Stage 1.

| Component | Selection and limitations | Cost target |
|---|---|---|
| Frontend | Vercel Hobby, only for this personal, non-commercial test. The current plan lists 4 active CPU-hours, 360 GB-hours provisioned memory and 1 million function invocations. The API proxy consumes function resources. Exceeding a quota can suspend a feature; no paid plan is enabled. [Official Hobby limits](https://vercel.com/docs/plans/hobby) | ₹0 within included use |
| Backend | One Render Free web instance: 0.1 CPU/512 MB. The workspace shares 750 instance-hours/month. Services sleep after 15 idle minutes and may take roughly a minute to wake. Filesystem is ephemeral; restarts can occur. No persistent disk/free worker/private inbound networking. SMTP ports are restricted; Stage 1 uses the portal. Keep other free services stopped and do not add a card or upgrade when exhausted. [Free service rules](https://render.com/docs/free), [instance specification](https://render.com/docs/blueprint-spec) | ₹0 for one free instance; unavailable when quota is exhausted |
| Database | Neon Free: 100 CU-hours/project/month and 5 GB public transfer. Scale-to-zero after five idle minutes is mandatory. Current published storage descriptions have changed; conservatively budget **500 MB**, and verify the account's displayed cap before deployment. Use pooled TLS Postgres and four application connections. At 0.25 CU, 100 CU-hours corresponds to 400 active hours, not an always-on month. [Plans and quota behavior](https://neon.com/docs/introduction/plans) | ₹0 within the actual account quota |
| AI | Deterministic summary/rules and chunk-level full-text extraction, clearly labeled as non-generative. No model API is necessary. `BETA_MODE=true` disables paid model use even if a key is accidentally supplied. | ₹0 inference-provider charges |
| Automation | Runs inside the triggering transaction, bounded to 32 nodes. State and failures persist. Notifications are projected on demand. No sleeping-host timers, cron, Kafka or permanent polling worker. | No extra host |

Memory fit and cold-start duration are **not verified on Render**. A later local Linux Docker rehearsal passed the constrained beta suites at 512 MiB/0.1 CPU, with about 70.6 MiB recorded cgroup peak for this small workload. This does not establish hosted Render behavior or sustained capacity. See STAGE1_SECURITY_CLOSURE.md for the completed scoped security assessment and STAGE1_FINAL_VERIFICATION.md for historical measurements. The combined container has one Rust server, one Python worker, no ML model in memory, no embeddings runtime and a bounded DB pool. If the actual container exceeds 512 MB or free account eligibility fails, stop and use the local Compose/native test alternative; do not upgrade.

## Alternatives evaluated

- **Koyeb Free:** one 512 MB/0.1 vCPU/2 GB instance per organization, Frankfurt or Washington; sleeps after one idle hour, no worker role or persistent volume. Its documented account verification can require a credit card, conflicting with this release's no-payment-method condition. Not selected. [Instances](https://www.koyeb.com/docs/reference/instances), [billing FAQ](https://www.koyeb.com/docs/faqs/pricing).
- **Supabase Free:** Postgres with 500 MB DB size and 5 GB egress; free projects can pause after a week of inactivity. Its pgvector extension is available. A valid alternative after checking database connection compatibility/quotas. It does not require replacing existing Rust auth with Supabase auth. [Pricing](https://supabase.com/pricing), [pgvector](https://supabase.com/docs/guides/database/extensions/pgvector).
- **Render Free Postgres:** expires after 30 days, so it is not the persistent testing database selection. [Expiry limitation](https://render.com/docs/free).
- **Groq Free inference:** legitimate but model/account-specific request/token quotas apply; 429 handling and explicit opt-in are essential. Not required, provisioned, or runtime-tested here. A supplied key does not mean unlimited free inference. [Rate limits](https://console.groq.com/docs/rate-limits).
- **Local Docker/native testing:** no new cloud bill, but uses the operator's existing hardware, power and network. This is the safe fallback, not a claim of free hosted public availability.

Neon also supports [pgvector](https://neon.com/docs/extensions/pgvector). Stage 1 deliberately uses indexed chunks/full-text retrieval without inventing embeddings or making a free inference dependency mandatory. A later semantic adapter can use the extension without replacing the document/tenant model.

## Operating boundaries

No keep-awake bots or synthetic traffic to defeat sleep. Do not enable Kafka, timers, continuously polling jobs, live email, attachments, payment processing or paid model access in the free profile. Cold-start/retry UI must never present a provider loading page as a successful JSON API operation. The frontend proxy returns an explicit temporary-unavailability response instead. Do not automatically replay writes after uncertain network failures.

The public beta accepts synthetic `.test` emails, caps organization creation, limits article size/count and leaves real-data collection unsupported. Storage/compute/egress must be monitored manually. Free quotas can change; recheck the linked primary documentation and dashboard immediately before a separately authorized deployment. If a provider asks for a payment method or cannot hard-stop usage at free limits, do not proceed.

Git-triggered deployments are disabled in both root/frontend Vercel config and the Render manifest. This branch is preparation only. [Vercel's documented disable switch](https://vercel.com/docs/project-configuration/git-configuration), [Render auto-deploy controls](https://render.com/docs/blueprint-spec).
