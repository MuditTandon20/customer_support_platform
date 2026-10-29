"use client";
import { useState } from "react";
import Link from "next/link";
import { useResource, State } from "./WorkspaceUI";
import { write, errorMessage } from "@/lib/platform";
interface Notification {
  id: string;
  resource_id: string;
  kind: string;
  read: boolean;
  created_at: string;
}
export default function Notifications() {
  const [open, setOpen] = useState(false);
  return (
    <div className="notification-control">
      <button
        aria-expanded={open}
        aria-label="Notifications"
        onClick={() => setOpen((v) => !v)}
      >
        ♧
      </button>
      {open && <NotificationList close={() => setOpen(false)} />}
    </div>
  );
}
function NotificationList({ close }: { close: () => void }) {
  const r = useResource<Notification[]>("/notifications");
  const [error, setError] = useState("");
  return (
    <section className="notification-popover" aria-label="Notifications">
      <div className="row">
        <h2>Workspace activity</h2>
        <button aria-label="Close notifications" onClick={close}>
          ✕
        </button>
      </div>
      {error && <p role="alert">{error}</p>}
      <State loading={r.loading} error={r.error} empty={!r.data?.length}>
        {r.data?.length ? (
          <div className="stack">
            {r.data.map((n) => (
              <article key={n.id} className={n.read ? "read" : ""}>
                <Link href={`/tickets/${n.resource_id}`} onClick={close}>
                  {n.kind}
                  <small>{new Date(n.created_at).toLocaleString()}</small>
                </Link>
                {!n.read && (
                  <button
                    aria-label={`Mark ${n.kind} as read`}
                    onClick={async () => {
                      try {
                        await write(`/notifications/${n.id}/read`);
                        r.refresh();
                      } catch (e) {
                        setError(errorMessage(e));
                      }
                    }}
                  >
                    ✓
                  </button>
                )}
              </article>
            ))}
          </div>
        ) : (
          <>No activity yet.</>
        )}
      </State>
    </section>
  );
}
