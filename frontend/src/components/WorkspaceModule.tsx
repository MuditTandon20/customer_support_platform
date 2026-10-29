"use client";
import { useState } from "react";
import Link from "next/link";
import {
  write,
  remove,
  errorMessage,
  type InboxTicket,
  type Article,
  type Analytics,
  type Action,
  type Integration,
  type Billing,
  type OrganizationSettings,
  type AuditEvent,
} from "@/lib/platform";
import { useUser } from "./WorkspaceShell";
import { useResource, Heading, State, Notice, Pill, date } from "./WorkspaceUI";
import EmailSandbox from "./EmailSandbox";
import { WorkflowStudio } from "./WorkflowStudio";

export function TicketTable({ tickets }: { tickets: InboxTicket[] }) {
  return (
    <div className="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Conversation</th>
            <th>Customer</th>
            <th>Status</th>
            <th>Priority</th>
            <th>Updated</th>
          </tr>
        </thead>
        <tbody>
          {tickets.map((t) => (
            <tr key={t.id}>
              <td>
                <Link className="ticket-link" href={`/tickets/${t.id}`}>
                  <span className="ticket-id">#{t.id.slice(0, 6)}</span>
                  {t.subject}
                </Link>
                <div className="label-row">
                  {t.labels?.map((l) => (
                    <span key={l}>{l}</span>
                  ))}
                </div>
              </td>
              <td>
                <strong>{t.customer_name}</strong>
                <small>{t.customer_email}</small>
              </td>
              <td>
                <Pill
                  tone={
                    ["solved", "closed"].includes(t.status)
                      ? "green"
                      : t.status === "pending"
                        ? "amber"
                        : "violet"
                  }
                >
                  {t.status.replace("_", " ")}
                </Pill>
              </td>
              <td>
                <span className={`priority ${t.priority}`}>●</span> {t.priority}
              </td>
              <td className="muted">{date(t.updated_at || t.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Dashboard() {
  const a = useResource<Analytics>("/analytics"),
    t = useResource<InboxTicket[]>("/inbox");
  const user = useUser();
  return (
    <>
      <Heading
        eyebrow="YOUR OPERATIONS, IN FOCUS"
        title="A little clarity. A lot of progress."
        description={`Welcome to ${user?.org_name || "your workspace"}. Here's where your support stands.`}
      >
        <Link className="button secondary" href="/workspace/analytics">
          View analytics ↗
        </Link>
        <Link className="button primary" href="/tickets/new">
          ＋ New conversation
        </Link>
      </Heading>
      <div className="insight-banner">
        <span className="insight-icon">✧</span>
        <div>
          <strong>Better answers start with better knowledge.</strong>
          <p>
            Add your help articles, then generate drafts with traceable
            evidence. You decide what gets sent.
          </p>
        </div>
        <Link href="/workspace/knowledge">Build your knowledge base →</Link>
      </div>
      <State loading={a.loading} error={a.error}>
        {a.data && (
          <>
            <div className="metric-grid">
              {[
                [
                  "Open backlog",
                  a.data.totals.backlog,
                  "Waiting for your team",
                  "violet",
                ],
                [
                  "Resolved tickets",
                  a.data.totals.resolved,
                  "Agent-set solved or closed",
                  "green",
                ],
                [
                  "Needs assignment",
                  a.data.totals.unassigned,
                  "Open and unassigned",
                  "cyan",
                ],
                [
                  "Older than 24 hours",
                  a.data.totals.over_24h,
                  "Backlog age, not an SLA",
                  "amber",
                ],
              ].map(([label, value, help, color]) => (
                <div className="metric-card" key={label}>
                  <div>
                    <span>{label}</span>
                    <span className={`metric-icon ${color}`}>◈</span>
                  </div>
                  <strong>{value}</strong>
                  <small>{help}</small>
                </div>
              ))}
            </div>
            <div className="dashboard-grid">
              <section className="panel">
                <div className="panel-heading">
                  <div>
                    <h2>Conversation activity</h2>
                    <p>Tickets created · last 14 days</p>
                  </div>
                  <Pill>Live data</Pill>
                </div>
                <ActivityChart data={a.data.daily} />
              </section>
              <section className="panel">
                <div className="panel-heading">
                  <div>
                    <h2>Your copilot, grounded</h2>
                    <p>Evidence-backed assistance</p>
                  </div>
                  <span className="violet">✧</span>
                </div>
                <div className="copilot-stat">
                  <strong>{a.data.ai.proposals}</strong>
                  <span>draft proposals</span>
                </div>
                <div className="split-stat">
                  <span>Cited proposals</span>
                  <strong>{a.data.ai.cited}</strong>
                </div>
                <div className="split-stat">
                  <span>Abstained without evidence</span>
                  <strong>{a.data.ai.abstained}</strong>
                </div>
                <div className="safe-note">
                  ♧ Extractive retrieval · No paid model calls
                </div>
                <Link href="/workspace/approvals" className="text-link">
                  Review action requests →
                </Link>
              </section>
            </div>
          </>
        )}
      </State>
      <section className="panel">
        <div className="panel-heading">
          <div>
            <h2>Latest conversations</h2>
            <p>Where your attention can make a difference</p>
          </div>
          <Link className="text-link" href="/workspace/inbox">
            Open inbox →
          </Link>
        </div>
        <State loading={t.loading} error={t.error} empty={t.data?.length === 0}>
          {t.data?.length ? (
            <TicketTable tickets={t.data.slice(0, 6)} />
          ) : (
            <>Create your first conversation to bring this workspace to life.</>
          )}
        </State>
      </section>
      <div className="quick-links">
        <Link href="/workspace/workflows">
          <span>⌘</span>
          <div>
            <strong>Let the routine run itself</strong>
            <small>Build controlled, inspectable workflows.</small>
          </div>
          ↗
        </Link>
        <Link href="/workspace/integrations">
          <span>⊞</span>
          <div>
            <strong>Connect on your terms</strong>
            <small>See exactly which channels are available.</small>
          </div>
          ↗
        </Link>
      </div>
    </>
  );
}
function ActivityChart({ data }: { data: Analytics["daily"] }) {
  const max = Math.max(1, ...data.map((d) => d.count));
  return (
    <div
      className="activity-chart"
      role="img"
      aria-label={`Daily tickets: ${data.map((d) => `${d.date}: ${d.count}`).join(", ")}`}
    >
      <div className="chart-bars">
        {data.map((d) => (
          <div className="chart-column" key={d.date}>
            <span>{d.count}</span>
            <div
              style={{ height: `${Math.max(2, (d.count / max) * 130)}px` }}
              title={`${d.date}: ${d.count} tickets`}
            />
            <small>{new Date(d.date).getUTCDate()}</small>
          </div>
        ))}
      </div>
      <div className="chart-caption">
        <span>Actual ticket volume</span>
        <span>14 days · UTC</span>
      </div>
    </div>
  );
}
function Inbox() {
  const r = useResource<InboxTicket[]>("/inbox");
  const [q, setQ] = useState(""),
    [status, setStatus] = useState("all");
  const user = useUser();
  const filtered =
    r.data?.filter(
      (t) =>
        (status === "all" ||
          (status === "unassigned"
            ? !t.assigned_to
            : status === "active"
              ? !["closed", "solved"].includes(t.status)
              : t.status === status)) &&
        `${t.subject} ${t.customer_name} ${t.customer_email} ${t.labels.join(" ")}`
          .toLowerCase()
          .includes(q.toLowerCase()),
    ) || [];
  return (
    <>
      <Heading
        eyebrow="CONVERSATIONS"
        title={
          user?.role === "customer"
            ? "Your support, in one place."
            : "The inbox. In sync."
        }
        description="Every conversation has a person behind it. Pick up where they left off."
      >
        <button onClick={r.refresh} className="secondary">
          ↻ Refresh
        </button>
        <Link className="button primary" href="/tickets/new">
          ＋ New conversation
        </Link>
      </Heading>
      <section className="panel">
        <div className="filter-bar">
          <div className="tabs" role="group" aria-label="Conversation status">
            {["all", "active", "pending", "solved", "unassigned"].map((s) => (
              <button
                key={s}
                className={status === s ? "selected" : ""}
                onClick={() => setStatus(s)}
              >
                {s === "all" ? "All conversations" : s}
              </button>
            ))}
          </div>
          <label className="search-field">
            <span className="sr-only">Search conversations</span>
            <input
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Search subjects, people, labels…"
            />
          </label>
        </div>
        <State loading={r.loading} error={r.error} empty={!filtered.length}>
          {filtered.length ? (
            <TicketTable tickets={filtered} />
          ) : (
            <>
              No matching conversations. Try another filter or create a
              conversation.
            </>
          )}
        </State>
        <div className="panel-footer">
          {filtered.length} conversations · Up to 250 most recently updated
        </div>
      </section>
    </>
  );
}
function Knowledge() {
  const r = useResource<Article[]>("/knowledge"),
    [q, setQ] = useState(""),
    [open, setOpen] = useState(false),
    [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false);
  const user = useUser();
  const [selected, setSelected] = useState<Article | null>(null);
  const articles =
    r.data?.filter((d) =>
      `${d.title} ${d.content}`.toLowerCase().includes(q.toLowerCase()),
    ) || [];
  async function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    const f = new FormData(e.currentTarget);
    try {
      await write("/knowledge", {
        title: f.get("title"),
        content: f.get("content"),
        visibility: f.get("visibility"),
        expires_at: f.get("expires_at")
          ? new Date(String(f.get("expires_at"))).toISOString()
          : null,
      });
      setNotice("Article indexed and ready for retrieval.");
      setOpen(false);
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function archive(a: Article) {
    if (
      !confirm(
        `Archive “${a.title}”? It will stop appearing in retrieval and the help center.`,
      )
    )
      return;
    try {
      await remove(`/knowledge/${a.id}`);
      setNotice("Article archived.");
      setSelected(null);
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    }
  }
  return (
    <>
      <Heading
        eyebrow="KNOWLEDGE INTELLIGENCE"
        title="Give every answer a source."
        description="Current, permitted knowledge. Traceable citations. A clear handoff when evidence is missing."
      >
        {user?.role !== "customer" && (
          <button className="primary" onClick={() => setOpen((v) => !v)}>
            ＋ Add article
          </button>
        )}
      </Heading>
      <Notice text={notice} />
      {open && (
        <form className="panel form-panel" onSubmit={submit}>
          <h2>Add a knowledge article</h2>
          <label>
            Title
            <input name="title" required maxLength={200} />
          </label>
          <label>
            Article content
            <textarea name="content" required maxLength={100000} rows={7} />
          </label>
          <div className="form-grid">
            <label>
              Visibility
              <select name="visibility">
                <option value="team">Team only</option>
                <option value="public">Public help center</option>
              </select>
            </label>
            <label>
              Fresh until (optional)
              <input name="expires_at" type="date" />
            </label>
          </div>
          <p className="muted">
            Indexed synchronously with PostgreSQL full-text search. Content
            remains untrusted evidence; it cannot authorize tools.
          </p>
          <div className="row">
            <button className="primary" disabled={busy}>
              {busy ? "Indexing…" : "Save and index"}
            </button>
            <button type="button" onClick={() => setOpen(false)}>
              Cancel
            </button>
          </div>
        </form>
      )}
      <div className="section-toolbar">
        <label className="search-field">
          <span className="sr-only">Search knowledge</span>
          <input
            placeholder="Search your knowledge…"
            value={q}
            onChange={(e) => setQ(e.target.value)}
          />
        </label>
        <span className="muted">{articles.length} current articles</span>
      </div>
      <State loading={r.loading} error={r.error} empty={!articles.length}>
        {articles.length ? (
          <div className="article-grid">
            {articles.map((a) => (
              <article className="panel article-card" key={a.id}>
                <div className="row">
                  <span className="article-icon">▧</span>
                  <Pill tone={a.visibility === "public" ? "green" : "violet"}>
                    {a.visibility}
                  </Pill>
                </div>
                <h2>
                  <button
                    className="link-button"
                    onClick={() => setSelected(a)}
                  >
                    {a.title}
                  </button>
                </h2>
                <p>
                  {a.content.slice(0, 180)}
                  {a.content.length > 180 ? "…" : ""}
                </p>
                <div className="article-meta">
                  <span>Updated {date(a.updated_at)}</span>
                  <span>v{a.version} · Indexed</span>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <>
            Add an approved article to enable cited drafts. No content is
            imported automatically.
          </>
        )}
      </State>
      {selected && (
        <section className="panel form-panel">
          <div className="row">
            <h2>{selected.title}</h2>
            <button
              onClick={() => setSelected(null)}
              aria-label="Close article"
            >
              ✕
            </button>
          </div>
          <div className="prose-text">{selected.content}</div>
          {user?.role !== "customer" && (
            <div className="row">
              <button
                onClick={async () => {
                  try {
                    await write(`/knowledge/${selected.id}/reindex`);
                    setNotice("Index refreshed.");
                    r.refresh();
                  } catch (e) {
                    setNotice(errorMessage(e));
                  }
                }}
              >
                ↻ Reindex
              </button>
              <button className="danger" onClick={() => archive(selected)}>
                Archive article
              </button>
            </div>
          )}
        </section>
      )}
    </>
  );
}
function Approvals() {
  const r = useResource<Action[]>("/approvals");
  const [notice, setNotice] = useState(""),
    [busy, setBusy] = useState("");
  const user = useUser();
  async function decide(id: string, approve: boolean) {
    setBusy(id);
    try {
      const result = await write<{ state: string }>(`/approvals/${id}`, {
        approve,
      });
      setNotice(
        approve
          ? `Approval recorded. Execution ${result.state}: provider adapter is not connected.`
          : "Request rejected.",
      );
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy("");
    }
  }
  return (
    <>
      <Heading
        eyebrow="HUMAN IN THE LOOP"
        title="Your judgment. Before every impact."
        description="Sensitive requests require an administrator. Approval never bypasses provider permissions."
      />
      <Notice text={notice} />
      <State loading={r.loading} error={r.error} empty={!r.data?.length}>
        {r.data?.length ? (
          <div className="stack">
            {r.data.map((a) => (
              <article className="panel action-card" key={a.id}>
                <div>
                  <div className="row">
                    <h2>{a.action}</h2>
                    <Pill
                      tone={
                        a.state === "pending"
                          ? "amber"
                          : a.state === "executed"
                            ? "green"
                            : "neutral"
                      }
                    >
                      {a.state}
                    </Pill>
                  </div>
                  <p>
                    {Object.entries(a.arguments)
                      .map(([k, v]) => `${k}: ${v}`)
                      .join(" · ") || "No arguments"}
                  </p>
                  <Link className="text-link" href={`/tickets/${a.ticket_id}`}>
                    Conversation #{a.ticket_id.slice(0, 6)} ↗
                  </Link>
                  <p className="muted">
                    {date(a.created_at)} ·{" "}
                    {a.state === "blocked"
                      ? "Approved, but external execution unavailable."
                      : "Execution is recorded separately from approval."}
                  </p>
                </div>
                {a.state === "pending" && user?.role === "admin" && (
                  <div className="row">
                    <button
                      disabled={busy === a.id}
                      onClick={() => decide(a.id, false)}
                    >
                      Reject
                    </button>
                    <button
                      className="primary"
                      disabled={busy === a.id}
                      onClick={() => decide(a.id, true)}
                    >
                      Record approval
                    </button>
                  </div>
                )}
              </article>
            ))}
          </div>
        ) : (
          <>
            There are no action requests. Propose an action from a conversation
            to start a review.
          </>
        )}
      </State>
    </>
  );
}
function AnalyticsPage() {
  const r = useResource<Analytics>("/analytics");
  function download() {
    if (!r.data) return;
    const blob = new Blob([JSON.stringify(r.data, null, 2)], {
        type: "application/json",
      }),
      url = URL.createObjectURL(blob),
      a = document.createElement("a");
    a.href = url;
    a.download = "support-analytics.json";
    a.click();
    URL.revokeObjectURL(url);
  }
  return (
    <>
      <Heading
        eyebrow="MEASURE WHAT HAPPENED"
        title="Clarity, backed by events."
        description="Real workspace records. Explicit definitions. No invented savings or resolution claims."
      >
        <button onClick={r.refresh}>↻ Refresh</button>
        <button className="primary" onClick={download} disabled={!r.data}>
          Export data ↓
        </button>
      </Heading>
      <State loading={r.loading} error={r.error}>
        {r.data && (
          <>
            <div className="metric-grid">
              {[
                ["Total conversations", r.data.totals.total],
                [
                  "First response",
                  r.data.first_response_minutes === null
                    ? "No replies"
                    : `${r.data.first_response_minutes.toFixed(1)} min`,
                ],
                [
                  "CSAT average",
                  r.data.csat_average === null
                    ? "No ratings"
                    : `${r.data.csat_average.toFixed(1)} / 5`,
                ],
                [
                  "Cited draft coverage",
                  r.data.ai.proposals
                    ? `${Math.round((r.data.ai.cited / r.data.ai.proposals) * 100)}%`
                    : "No drafts",
                ],
              ].map(([label, value]) => (
                <div className="metric-card" key={label}>
                  <span>{label}</span>
                  <strong>{value}</strong>
                  <small>Calculated from stored records</small>
                </div>
              ))}
            </div>
            <div className="dashboard-grid">
              <section className="panel">
                <div className="panel-heading">
                  <h2>Conversation volume</h2>
                </div>
                <ActivityChart data={r.data.daily} />
              </section>
              <section className="panel">
                <div className="panel-heading">
                  <h2>Status distribution</h2>
                </div>
                <div className="status-chart">
                  {r.data.statuses.length ? (
                    r.data.statuses.map((s) => (
                      <div key={s.status}>
                        <div className="row">
                          <span>{s.status}</span>
                          <strong>{s.count}</strong>
                        </div>
                        <progress
                          max={r.data?.totals.total || 1}
                          value={s.count}
                          aria-label={s.status}
                        />
                      </div>
                    ))
                  ) : (
                    <p>No conversations yet.</p>
                  )}
                </div>
              </section>
            </div>
            <section className="panel form-panel">
              <h2>What these numbers mean</h2>
              {Object.entries(r.data.definitions).map(([k, v]) => (
                <div className="definition" key={k}>
                  <strong>{k.replaceAll("_", " ")}</strong>
                  <p>{v}</p>
                </div>
              ))}
            </section>
          </>
        )}
      </State>
    </>
  );
}
function Customers() {
  const r = useResource<InboxTicket[]>("/inbox");
  const [q, setQ] = useState("");
  const customers = new Map<
    string,
    { name: string; email: string; count: number; open: number; id: string }
  >();
  r.data?.forEach((t) => {
    const c = customers.get(t.created_by) || {
      name: t.customer_name,
      email: t.customer_email,
      count: 0,
      open: 0,
      id: t.id,
    };
    c.count++;
    if (!["solved", "closed"].includes(t.status)) c.open++;
    customers.set(t.created_by, c);
  });
  return (
    <>
      <Heading
        eyebrow="THE PEOPLE BEHIND THE TICKETS"
        title="A little more context. A better conversation."
        description="Customer profiles from the 250 most recent conversations in this workspace."
      />
      <label className="search-field">
        <span className="sr-only">Find a customer</span>
        <input
          placeholder="Search names or email…"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
      </label>
      <State loading={r.loading} error={r.error} empty={!customers.size}>
        {customers.size ? (
          <div className="article-grid">
            {[...customers]
              .filter(([, c]) =>
                `${c.name} ${c.email}`.toLowerCase().includes(q.toLowerCase()),
              )
              .map(([id, c]) => (
                <article className="panel customer-card" key={id}>
                  <span className="avatar">{c.name.slice(0, 1)}</span>
                  <h2>{c.name}</h2>
                  <p>{c.email}</p>
                  <div className="split-stat">
                    <span>Recent conversations</span>
                    <strong>{c.count}</strong>
                  </div>
                  <div className="split-stat">
                    <span>Open conversations</span>
                    <strong>{c.open}</strong>
                  </div>
                  <Link className="text-link" href={`/tickets/${c.id}`}>
                    Open latest conversation ↗
                  </Link>
                </article>
              ))}
          </div>
        ) : (
          <>Customer context appears when a conversation is created.</>
        )}
      </State>
    </>
  );
}
function Integrations() {
  const r = useResource<Integration[]>("/integrations");
  const user = useUser();
  return (
    <>
      <Heading
        eyebrow="YOUR CONNECTED WORKSPACE"
        title="Connections, with clear boundaries."
        description="An honest view of what's active, local, or waiting for a provider."
      />
      <State loading={r.loading} error={r.error}>
        <div className="article-grid">
          {r.data?.map((i, index) => (
            <article className="panel integration-card" key={i.id}>
              <div className="row">
                <span className="integration-icon">
                  {["▤", "✉", "◉", "✧", "▣"][index]}
                </span>
                <Pill
                  tone={
                    i.status === "active"
                      ? "green"
                      : i.status === "local"
                        ? "violet"
                        : "neutral"
                  }
                >
                  {i.status.replace("_", " ")}
                </Pill>
              </div>
              <h2>{i.name}</h2>
              <p>{i.detail}</p>
              <span className="muted">
                {i.status === "active"
                  ? "Available in this workspace"
                  : i.status === "local"
                    ? "No external credentials used"
                    : "Provider setup required"}
              </span>
            </article>
          ))}
        </div>
      </State>
      <section className="panel form-panel">
        <Link className="text-link" href="/workspace/sandbox">
          Open local email sandbox ?
        </Link>
        <h2>Embed the customer portal</h2>
        <p>
          The widget uses an authenticated portal. It does not expose an
          anonymous ticket API or internal notes.
        </p>
        <code className="code-block">{`<iframe src="${typeof window !== "undefined" ? window.location.origin : "http://localhost:3000"}/widget" title="Customer support" width="380" height="640"></iframe>`}</code>
        {user?.org_slug && (
          <Link className="text-link" href={`/help/${user.org_slug}`}>
            Open public help center ↗
          </Link>
        )}
      </section>
    </>
  );
}
function BillingPage() {
  const r = useResource<Billing>("/billing");
  return (
    <>
      <Heading
        eyebrow="USAGE & ENTITLEMENTS"
        title="Room to build. Limits you can see."
        description="Local trial entitlements. No payment provider is enabled and no charges can be made."
      />
      <State loading={r.loading} error={r.error}>
        {r.data && (
          <>
            <section className="panel plan-card">
              <div>
                <Pill tone="violet">{r.data.plan}</Pill>
                <h2>Your local workspace</h2>
                <p>
                  Payments disabled · No card on file · No invoices generated
                </p>
              </div>
              <strong>
                ₹0<small>provider charges</small>
              </strong>
            </section>
            <div className="article-grid">
              {Object.entries(r.data.limits).map(([k, v]) => (
                <article className="panel usage-card" key={k}>
                  <h2>{k.replaceAll("_", " ")}</h2>
                  <strong>
                    {r.data?.usage[k] || 0}
                    <span> / {v}</span>
                  </strong>
                  <progress
                    max={v}
                    value={r.data?.usage[k] || 0}
                    aria-label={k}
                  />
                </article>
              ))}
            </div>
            <div className="safe-note">
              Trial counters are enforced by Rust. Daily limits reset at 00:00
              UTC. Infrastructure costs are outside this view.
            </div>
          </>
        )}
      </State>
    </>
  );
}
function Settings() {
  const r = useResource<OrganizationSettings>("/settings");
  const [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false),
    [invite, setInvite] = useState("");
  async function save(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    const f = new FormData(e.currentTarget);
    try {
      await write(
        "/settings",
        {
          name: f.get("name"),
          reply_tone: f.get("tone"),
          onboarding_complete: f.get("onboarded") === "on",
        },
        "PATCH",
      );
      setNotice("Workspace preferences saved.");
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function inviteMember(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setBusy(true);
    const f = new FormData(e.currentTarget);
    try {
      const result = await write<{ token: string }>("/invitations", {
        email: f.get("email"),
        role: f.get("role"),
      });
      setInvite(`${location.origin}/join?token=${result.token}`);
      setNotice(
        "Invitation created. Share the link securely; no email was sent.",
      );
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <Heading
        eyebrow="MAKE IT YOUR WORKSPACE"
        title="A thoughtful setup, from the start."
        description="Organization preferences, onboarding and invitation-only team access."
      />
      <Notice text={notice} />
      <State loading={r.loading} error={r.error}>
        {r.data && (
          <div className="settings-grid">
            <form
              className="panel form-panel"
              key={r.data.name}
              onSubmit={save}
            >
              <h2>Workspace details</h2>
              <label>
                Organization name
                <input
                  name="name"
                  required
                  maxLength={100}
                  defaultValue={r.data.name}
                />
              </label>
              <label>
                Preferred reply tone
                <select
                  name="tone"
                  defaultValue={r.data.settings.reply_tone || "professional"}
                >
                  <option>professional</option>
                  <option>friendly</option>
                  <option>concise</option>
                </select>
              </label>
              <p className="muted">
                Tone is stored for future generated replies. Extractive drafts
                preserve source wording.
              </p>
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  name="onboarded"
                  defaultChecked={r.data.settings.onboarding_complete}
                />{" "}
                Knowledge and team setup reviewed
              </label>
              <button disabled={busy} className="primary">
                Save preferences
              </button>
            </form>
            <form className="panel form-panel" onSubmit={inviteMember}>
              <h2>Invite a teammate or customer</h2>
              <label>
                Email address
                <input name="email" type="email" required />
              </label>
              <label>
                Workspace role
                <select name="role">
                  <option value="agent">Agent</option>
                  <option value="customer">Customer</option>
                </select>
              </label>
              <p className="muted">
                Single-use invitation · Expires in 48 hours · Requires the
                matching email.
              </p>
              <button disabled={busy}>Create invitation link</button>
              {invite && (
                <label>
                  Share this link securely
                  <input
                    readOnly
                    value={invite}
                    onFocus={(e) => e.target.select()}
                  />
                </label>
              )}
              <Link className="text-link" href="/tickets/admin">
                Manage team permissions →
              </Link>
            </form>
          </div>
        )}
      </State>
      <section className="panel form-panel">
        <h2>Finish your setup</h2>
        <div className="onboarding-list">
          <Link href="/workspace/knowledge">
            01 <strong>Add approved knowledge</strong>
            <span>Articles with clear visibility and freshness →</span>
          </Link>
          <Link href="/workspace/workflows">
            02 <strong>Create a controlled workflow</strong>
            <span>Simulate before publishing →</span>
          </Link>
          <Link href="/workspace/integrations">
            03 <strong>Review your channels</strong>
            <span>See exactly what is connected →</span>
          </Link>
        </div>
      </section>
    </>
  );
}
function Audit() {
  const r = useResource<AuditEvent[]>("/audit");
  return (
    <>
      <Heading
        eyebrow="ACCOUNTABILITY, BUILT IN"
        title="A record you can follow."
        description="Actor, resource, action and time. The latest 250 workspace events."
      >
        <button onClick={r.refresh}>↻ Refresh</button>
      </Heading>
      <State loading={r.loading} error={r.error} empty={!r.data?.length}>
        {r.data?.length ? (
          <section
            className="panel table-scroll"
            tabIndex={0}
            aria-label="Audit event table"
          >
            <table>
              <thead>
                <tr>
                  <th>Event</th>
                  <th>Actor</th>
                  <th>Resource</th>
                  <th>Time</th>
                </tr>
              </thead>
              <tbody>
                {r.data.map((e) => (
                  <tr key={e.id}>
                    <td>
                      <Pill>{e.action}</Pill>
                      <small>{JSON.stringify(e.detail)}</small>
                    </td>
                    <td className="mono">
                      {e.actor_id?.slice(0, 8) || "System"}
                    </td>
                    <td className="mono">{e.resource_id?.slice(0, 8)}</td>
                    <td>{new Date(e.created_at).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>
        ) : (
          <>No audited actions yet.</>
        )}
      </State>
    </>
  );
}
export default function WorkspaceModule({ module }: { module: string }) {
  const user = useUser();
  if (user?.role === "customer" && !["inbox", "knowledge"].includes(module))
    return <Inbox />;
  switch (module) {
    case "dashboard":
      return <Dashboard />;
    case "inbox":
      return <Inbox />;
    case "knowledge":
      return <Knowledge />;
    case "workflows":
      return <WorkflowStudio />;
    case "approvals":
      return <Approvals />;
    case "analytics":
      return <AnalyticsPage />;
    case "customers":
      return <Customers />;
    case "integrations":
      return <Integrations />;
    case "sandbox":
      return <EmailSandbox />;
    case "billing":
      return <BillingPage />;
    case "settings":
      return <Settings />;
    case "audit":
      return <Audit />;
    default:
      return (
        <div className="empty-state">
          <h1>Page not found</h1>
          <Link href="/workspace/dashboard">Return to overview</Link>
        </div>
      );
  }
}
