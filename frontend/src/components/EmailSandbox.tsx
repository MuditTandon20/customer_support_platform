"use client";
import Link from "next/link";
import { useState } from "react";
import { useResource, Heading, State, Notice, Pill } from "./WorkspaceUI";
import { write, key, errorMessage } from "@/lib/platform";
interface Mail {
  id: string;
  ticket_id: string | null;
  recipient: string;
  subject: string;
  body: string;
  direction: string;
  state: string;
  created_at: string;
}
export default function EmailSandbox() {
  const r = useResource<Mail[]>("/sandbox/mailbox");
  const [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false);
  return (
    <>
      <Heading
        eyebrow="LOCAL EMAIL SANDBOX"
        title="Test the round trip. Without sending an email."
        description="Administrator-only test mailbox. Requires ENABLE_SANDBOX=true. All messages stay in your database."
      >
        <button onClick={r.refresh}>↻ Refresh mailbox</button>
      </Heading>
      <Notice text={notice} />
      <div className="safe-note">
        This mailbox contains local verification and password reset links. Treat
        them as secrets. Disable sandbox mode in production.
      </div>
      <form
        className="panel form-panel"
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          const f = new FormData(e.currentTarget);
          try {
            const mail = await write<Mail>("/sandbox/email", {
              customer_email: f.get("email"),
              subject: f.get("subject"),
              body: f.get("body"),
              ticket_id: f.get("ticket") || null,
              idempotency_key: key(),
            });
            setNotice(
              `Inbound sandbox email saved. Conversation #${mail.ticket_id?.slice(0, 8)} is now in the inbox.`,
            );
            r.refresh();
          } catch (e) {
            setNotice(errorMessage(e));
          } finally {
            setBusy(false);
          }
        }}
      >
        <h2>Simulate an inbound email</h2>
        <p>
          First invite and register a customer. Reply to the resulting
          conversation to see the outbound sandbox message here.
        </p>
        <div className="form-grid">
          <label>
            Registered customer email
            <input name="email" type="email" required />
          </label>
          <label>
            Subject
            <input name="subject" required maxLength={200} />
          </label>
        </div>
        <label>
          Message
          <textarea name="body" rows={4} required maxLength={20000} />
        </label>
        <label>
          Existing sandbox conversation ID (optional)
          <input
            name="ticket"
            placeholder="Leave blank to create a conversation"
          />
        </label>
        <button className="primary" disabled={busy}>
          Simulate inbound email
        </button>
      </form>
      <State loading={r.loading} error={r.error} empty={!r.data?.length}>
        {r.data?.length ? (
          <div className="stack">
            {r.data.map((m) => (
              <article className="panel form-panel" key={m.id}>
                <div className="row">
                  <h2>{m.subject}</h2>
                  <Pill tone="amber">{m.direction} · Sandbox only</Pill>
                </div>
                <p>
                  {m.recipient} · {new Date(m.created_at).toLocaleString()}
                </p>
                <div className="prose-text">
                  {m.body.split(/(https?:\/\/\S+)/).map((part, i) =>
                    part.startsWith("http") ? (
                      <a className="text-link" href={part} key={i}>
                        {part}
                      </a>
                    ) : (
                      part
                    ),
                  )}
                </div>
                {m.ticket_id && (
                  <Link className="text-link" href={`/tickets/${m.ticket_id}`}>
                    Open conversation ↗
                  </Link>
                )}
              </article>
            ))}
          </div>
        ) : (
          <>No messages in this sandbox mailbox yet.</>
        )}
      </State>
    </>
  );
}
