"use client";
import Link from "next/link";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { write, errorMessage } from "@/lib/platform";
import { saveToken } from "@/lib/auth";
export default function Join() {
  const [error, setError] = useState(""),
    [busy, setBusy] = useState(false);
  const router = useRouter();
  return (
    <main className="workspace public-page">
      <Link className="brand" href="/">
        aster <span className="brand-caption">SUPPORT OS</span>
      </Link>
      <form
        className="panel form-panel auth-form"
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          const f = new FormData(e.currentTarget);
          try {
            const r = await write<{ token: string }>(
              "/auth/accept-invitation",
              {
                token:
                  new URLSearchParams(location.search).get("token") ||
                  f.get("token"),
                email: f.get("email"),
                password: f.get("password"),
                full_name: f.get("name"),
              },
            );
            saveToken(r.token);
            router.push("/workspace/inbox");
          } catch (e) {
            setError(errorMessage(e));
          } finally {
            setBusy(false);
          }
        }}
      >
        <p className="eyebrow">YOU&apos;RE INVITED</p>
        <h1>Your next workspace awaits.</h1>
        <p>Use the email address this invitation was issued to.</p>
        {error && (
          <div role="alert" className="error-banner">
            {error}
          </div>
        )}
        <label>
          Full name
          <input required name="name" autoComplete="name" />
        </label>
        <label>
          Email
          <input required name="email" type="email" autoComplete="email" />
        </label>
        <label>
          Password
          <input
            required
            name="password"
            type="password"
            minLength={12}
            maxLength={256}
            autoComplete="new-password"
          />
        </label>
        <button className="primary" disabled={busy}>
          {busy ? "Joining…" : "Accept invitation"}
        </button>
      </form>
    </main>
  );
}
