"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { createTicket } from "@/lib/api-client";
import { errorMessage } from "@/lib/platform";
import { Heading, Notice } from "@/components/WorkspaceUI";
export default function NewConversation() {
  const [error, setError] = useState(""),
    [busy, setBusy] = useState(false);
  const router = useRouter();
  return (
    <>
      <Heading
        eyebrow="START A CONVERSATION"
        title="A good place to begin."
        description="Tell us what you need. Your conversation stays within your organization."
      />
      <Notice text={error} />
      <form
        className="panel form-panel"
        style={{ maxWidth: 800 }}
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          const f = new FormData(e.currentTarget);
          try {
            const t = await createTicket({
              subject: String(f.get("subject")),
              description: String(f.get("description")),
            });
            router.push(`/tickets/${t.id}`);
          } catch (e) {
            setError(errorMessage(e));
            setBusy(false);
          }
        }}
      >
        <label>
          Subject
          <input
            name="subject"
            required
            maxLength={200}
            placeholder="Brief description of the issue"
          />
        </label>
        <label>
          Description
          <textarea
            name="description"
            maxLength={20000}
            rows={9}
            placeholder="Provide details about the issue"
          />
        </label>
        <div className="row">
          <Link className="text-link" href="/workspace/inbox">
            Cancel
          </Link>
          <button disabled={busy} className="primary">
            {busy ? "Creating…" : "Create Ticket"}
          </button>
        </div>
      </form>
    </>
  );
}
