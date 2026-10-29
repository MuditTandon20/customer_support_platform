"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useUser } from "./WorkspaceShell";
import { useResource, State, Notice, Pill, Heading } from "./WorkspaceUI";
import {
  write,
  key,
  errorMessage,
  type Message,
  type InboxTicket,
  type Proposal,
} from "@/lib/platform";
import {
  getTicketSummary,
  getTicketCategory,
  updateTicket,
  listUsers,
} from "@/lib/api-client";
import type { TicketStatus, UserSummary } from "@/types";
export default function Conversation() {
  const { id } = useParams<{ id: string }>(),
    user = useUser(),
    r = useResource<InboxTicket>(`/tickets/${id}`),
    thread = useResource<Message[]>(`/tickets/${id}/messages`);
  const [body, setBody] = useState(""),
    [internal, setInternal] = useState(false),
    [busy, setBusy] = useState(false),
    [notice, setNotice] = useState(""),
    [proposal, setProposal] = useState<Proposal | null>(null),
    [summary, setSummary] = useState(""),
    [team, setTeam] = useState<UserSummary[]>([]),
    [tool, setTool] = useState("ticket.label"),
    [argument, setArgument] = useState("reviewed"),
    [preview, setPreview] = useState<Record<string, unknown> | null>(null);
  const [messageKey, setMessageKey] = useState("");
  const [proposalInEditor, setProposalInEditor] = useState<string | null>(null);
  useEffect(() => {
    if (user?.role !== "customer")
      listUsers()
        .then(setTeam)
        .catch(() => {});
  }, [user?.role]);
  const staff = user?.role !== "customer";
  async function send(e: React.FormEvent) {
    e.preventDefault();
    if (!body.trim()) return;
    setBusy(true);
    const idempotency = messageKey || key();
    setMessageKey(idempotency);
    try {
      await write(`/tickets/${id}/messages`, {
        body,
        internal,
        idempotency_key: idempotency,
        proposal_id: proposalInEditor,
      });
      setBody("");
      setMessageKey("");
      setProposalInEditor(null);
      setNotice(
        internal
          ? "Internal note saved."
          : "Reply is visible in the customer portal. No email was sent.",
      );
      thread.refresh();
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function status(value: string) {
    setBusy(true);
    try {
      await updateTicket(id, { status: value as TicketStatus });
      r.refresh();
      setNotice("Conversation status updated.");
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function draft() {
    setBusy(true);
    try {
      setProposal(await write<Proposal>(`/tickets/${id}/grounded-draft`));
      setNotice(
        "Evidence proposal created. Review the article excerpts before using it.",
      );
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  function toolBody() {
    return {
      ticket_id: id,
      action: tool,
      arguments:
        tool === "account.lookup"
          ? {}
          : tool === "ticket.label"
            ? { label: argument }
            : tool === "ticket.status"
              ? { status: argument }
              : { reason: argument },
      idempotency_key: key(),
    };
  }
  return (
    <>
      <Link className="back-link" href="/workspace/inbox">
        ← All conversations
      </Link>
      <State loading={r.loading} error={r.error}>
        {r.data && (
          <>
            <Heading
              eyebrow={`CONVERSATION #${id.slice(0, 8).toUpperCase()}`}
              title={r.data.subject}
              description={`Opened ${new Date(r.data.created_at).toLocaleString()} · Customer portal`}
            >
              <Pill tone="violet">{r.data.status.replace("_", " ")}</Pill>
              {staff && (
                <label>
                  <span className="sr-only">Conversation status</span>
                  <select
                    disabled={busy}
                    value={r.data.status}
                    onChange={(e) => status(e.target.value)}
                  >
                    {[
                      "new",
                      "open",
                      "pending",
                      "on_hold",
                      "solved",
                      "closed",
                    ].map((s) => (
                      <option key={s}>{s}</option>
                    ))}
                  </select>
                </label>
              )}
            </Heading>
            <Notice text={notice} />
            <div className="conversation-layout">
              <section className="panel conversation-thread">
                <div className="panel-heading">
                  <h2>Conversation</h2>
                  <button onClick={thread.refresh}>↻ Refresh</button>
                </div>
                <article className="message initial">
                  <div className="row">
                    <strong>Original request</strong>
                    <time>{new Date(r.data.created_at).toLocaleString()}</time>
                  </div>
                  <div className="prose-text">
                    {r.data.description || "No description provided."}
                  </div>
                </article>
                <State loading={thread.loading} error={thread.error}>
                  {thread.data?.map((m) => (
                    <article
                      key={m.id}
                      className={`message ${m.internal ? "internal" : m.author_role === "customer" ? "customer" : "agent"}`}
                    >
                      <div className="row">
                        <div>
                          <strong>{m.author_name}</strong>{" "}
                          <Pill tone={m.internal ? "amber" : "neutral"}>
                            {m.internal ? "Internal note" : m.author_role}
                          </Pill>
                        </div>
                        <time>{new Date(m.created_at).toLocaleString()}</time>
                      </div>
                      <div className="prose-text">{m.body}</div>
                      <small>
                        {m.internal
                          ? "Only visible to your team"
                          : "Visible in the customer portal"}
                      </small>
                    </article>
                  ))}
                </State>
                {r.data.status !== "closed" ? (
                  <form className="reply-composer" onSubmit={send}>
                    {staff && (
                      <div
                        className="tabs"
                        role="group"
                        aria-label="Message visibility"
                      >
                        <button
                          type="button"
                          className={!internal ? "selected" : ""}
                          onClick={() => {
                            setInternal(false);
                            setMessageKey("");
                          }}
                        >
                          Public reply
                        </button>
                        <button
                          type="button"
                          className={internal ? "selected" : ""}
                          onClick={() => {
                            setInternal(true);
                            setMessageKey("");
                          }}
                        >
                          Internal note
                        </button>
                      </div>
                    )}
                    <label htmlFor="reply-body">
                      {internal ? "Note for your team" : "Your reply"}
                    </label>
                    <textarea
                      id="reply-body"
                      required
                      maxLength={20000}
                      value={body}
                      onChange={(e) => {
                        setBody(e.target.value);
                        setMessageKey("");
                        setProposalInEditor(null);
                      }}
                      rows={6}
                      placeholder={
                        internal
                          ? "Add context for your team…"
                          : "Write a thoughtful reply…"
                      }
                    />
                    <div className="row">
                      <small>
                        {internal
                          ? "Hidden from customers"
                          : "Portal delivery · No external email provider"}
                      </small>
                      <button
                        className="primary"
                        disabled={busy || !body.trim()}
                      >
                        {busy
                          ? "Saving…"
                          : internal
                            ? "Save internal note"
                            : "Send portal reply ↗"}
                      </button>
                    </div>
                  </form>
                ) : (
                  <div className="safe-note">This conversation is closed.</div>
                )}
                {!staff && ["solved", "closed"].includes(r.data.status) && (
                  <div className="form-panel">
                    <h2>How did we do?</h2>
                    <div
                      className="row"
                      role="group"
                      aria-label="Rate support from 1 to 5"
                    >
                      {[1, 2, 3, 4, 5].map((score) => (
                        <button
                          key={score}
                          onClick={async () => {
                            try {
                              await write(`/tickets/${id}/csat`, { score });
                              setNotice("Thank you. Your feedback was saved.");
                            } catch (e) {
                              setNotice(errorMessage(e));
                            }
                          }}
                        >
                          {score} ★
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </section>
              <aside className="conversation-aside">
                {staff && (
                  <>
                    <section className="panel copilot-panel">
                      <div className="panel-heading">
                        <div>
                          <p className="eyebrow">EVIDENCE, BEFORE ANSWERS</p>
                          <h2>✧ Knowledge copilot</h2>
                        </div>
                        <Pill tone="violet">Extractive</Pill>
                      </div>
                      <div className="form-panel">
                        <p>
                          Retrieve current articles from this workspace. Review
                          relevance before composing a reply.
                        </p>
                        <button
                          className="primary full"
                          disabled={busy}
                          onClick={draft}
                        >
                          ✧ Find evidence & draft
                        </button>
                        {proposal && (
                          <div className="draft-card">
                            <Pill tone={proposal.abstained ? "amber" : "green"}>
                              {proposal.abstained
                                ? "Abstained · Missing evidence"
                                : "Suggested · Not sent"}
                            </Pill>
                            <div className="prose-text">{proposal.draft}</div>
                            <div className="citations">
                              {proposal.citations.map((c, i) => (
                                <details key={c.id}>
                                  <summary>
                                    [{i + 1}] {c.title} · v{c.version}
                                  </summary>
                                  <p>{c.excerpt}</p>
                                  <Link href="/workspace/knowledge">
                                    Review source library →
                                  </Link>
                                </details>
                              ))}
                            </div>
                            {!proposal.abstained && (
                              <button
                                onClick={() => {
                                  setBody(proposal.draft);
                                  setProposalInEditor(proposal.id);
                                  setInternal(false);
                                  setMessageKey("");
                                }}
                              >
                                Use in reply editor
                              </button>
                            )}
                          </div>
                        )}
                        <div className="row">
                          <button
                            disabled={busy}
                            onClick={async () => {
                              setBusy(true);
                              try {
                                const result = await getTicketSummary(id);
                                setSummary(`Summary: ${result.summary}`);
                              } catch (e) {
                                setNotice(errorMessage(e));
                              } finally {
                                setBusy(false);
                              }
                            }}
                          >
                            Summarize
                          </button>
                          <button
                            disabled={busy}
                            onClick={async () => {
                              setBusy(true);
                              try {
                                const result = await getTicketCategory(id);
                                setSummary(
                                  `Suggested category: ${result.category}. Priority: ${result.priority}. ${result.reasoning || ""}`,
                                );
                              } catch (e) {
                                setNotice(errorMessage(e));
                              } finally {
                                setBusy(false);
                              }
                            }}
                          >
                            Classify
                          </button>
                        </div>
                        {summary && (
                          <div className="safe-note">
                            Heuristic/local unless a model is explicitly
                            enabled.<p>{summary}</p>
                          </div>
                        )}
                      </div>
                    </section>
                    <section className="panel form-panel">
                      <h2>Conversation details</h2>
                      <label>
                        Assigned teammate
                        <select
                          disabled={busy}
                          value={r.data.assigned_to || ""}
                          onChange={async (e) => {
                            try {
                              await updateTicket(
                                id,
                                e.target.value
                                  ? { assigned_to: e.target.value }
                                  : { unassign: true },
                              );
                              r.refresh();
                              setNotice("Assignment saved.");
                            } catch (e) {
                              setNotice(errorMessage(e));
                            }
                          }}
                        >
                          <option value="">Unassigned</option>
                          {team
                            .filter((u) => u.role !== "customer")
                            .map((u) => (
                              <option key={u.id} value={u.id}>
                                {u.full_name}
                              </option>
                            ))}
                        </select>
                      </label>
                      <form
                        onSubmit={async (e) => {
                          e.preventDefault();
                          const f = new FormData(e.currentTarget);
                          try {
                            await write(
                              `/tickets/${id}/metadata`,
                              {
                                priority: f.get("priority"),
                                labels: String(f.get("labels"))
                                  .split(",")
                                  .map((v) => v.trim())
                                  .filter(Boolean),
                              },
                              "PATCH",
                            );
                            setNotice("Priority and labels saved.");
                            r.refresh();
                          } catch (e) {
                            setNotice(errorMessage(e));
                          }
                        }}
                      >
                        <label>
                          Priority
                          <select
                            name="priority"
                            defaultValue={r.data.priority || "normal"}
                          >
                            {["low", "normal", "high", "urgent"].map((p) => (
                              <option key={p}>{p}</option>
                            ))}
                          </select>
                        </label>
                        <label>
                          Labels, comma separated
                          <input
                            name="labels"
                            defaultValue={r.data.labels?.join(", ")}
                          />
                        </label>
                        <button>Save details</button>
                      </form>
                    </section>
                    <section className="panel form-panel">
                      <h2>Controlled actions</h2>
                      <label>
                        Action
                        <select
                          value={tool}
                          onChange={(e) => {
                            setTool(e.target.value);
                            setArgument(
                              e.target.value === "ticket.status"
                                ? "open"
                                : e.target.value === "ticket.label"
                                  ? "reviewed"
                                  : "",
                            );
                            setPreview(null);
                          }}
                        >
                          <option value="ticket.label">Add ticket label</option>
                          <option value="ticket.status">
                            Change ticket status
                          </option>
                          <option value="account.lookup">
                            Read customer account
                          </option>
                          <option value="refund">
                            Request refund approval
                          </option>
                          <option value="subscription.cancel">
                            Request cancellation approval
                          </option>
                          <option value="data.export">
                            Request export approval
                          </option>
                        </select>
                      </label>
                      {tool !== "account.lookup" && (
                        <label>
                          {tool === "ticket.label"
                            ? "Label"
                            : tool === "ticket.status"
                              ? "Status"
                              : "Reason"}
                          <input
                            value={argument}
                            onChange={(e) => {
                              setArgument(e.target.value);
                              setPreview(null);
                            }}
                            required
                          />
                        </label>
                      )}
                      <button
                        disabled={busy}
                        onClick={async () => {
                          try {
                            setPreview(
                              await write<Record<string, unknown>>(
                                "/tools/preview",
                                toolBody(),
                              ),
                            );
                          } catch (e) {
                            setNotice(errorMessage(e));
                          }
                        }}
                      >
                        Preview action
                      </button>
                      {preview && (
                        <div className="safe-note">
                          <p>
                            {preview.requires_admin_approval
                              ? "Administrator approval required. External execution is unavailable."
                              : "Allowlisted local action. Runs immediately when confirmed."}
                          </p>
                          <button
                            disabled={busy}
                            onClick={async () => {
                              setBusy(true);
                              try {
                                const result = await write<{
                                  state: string;
                                  result: Record<string, unknown>;
                                }>("/tools", toolBody());
                                setNotice(
                                  `Action ${result.state}. ${JSON.stringify(result.result)}`,
                                );
                                setPreview(null);
                                r.refresh();
                              } catch (e) {
                                setNotice(errorMessage(e));
                              } finally {
                                setBusy(false);
                              }
                            }}
                          >
                            Confirm request
                          </button>
                        </div>
                      )}
                      <Link className="text-link" href="/workspace/approvals">
                        View action history →
                      </Link>
                    </section>
                  </>
                )}
              </aside>
            </div>
          </>
        )}
      </State>
    </>
  );
}
