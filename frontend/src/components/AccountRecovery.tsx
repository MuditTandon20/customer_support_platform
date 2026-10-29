"use client";
import { useState } from "react";
import Link from "next/link";
import { write, errorMessage } from "@/lib/platform";
export default function AccountRecovery({
  mode,
}: {
  mode: "request" | "reset" | "verify";
}) {
  const [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false),
    [done, setDone] = useState(false);
  return (
    <main className="workspace public-page">
      <Link className="brand" href="/">
        <span className="brand-mark">a</span>aster
      </Link>
      <form
        className="panel form-panel auth-form"
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          const f = new FormData(e.currentTarget);
          const token =
            new URLSearchParams(location.search).get("token") || f.get("token");
          try {
            const result = await write<{ message?: string }>(
              mode === "request"
                ? "/auth/request-reset"
                : mode === "reset"
                  ? "/auth/reset-password"
                  : "/auth/verify-email",
              mode === "request"
                ? { email: f.get("email") }
                : mode === "reset"
                  ? { token, password: f.get("password") }
                  : { token },
            );
            setNotice(
              result.message ||
                (mode === "reset"
                  ? "Password updated. Previous sessions have been revoked."
                  : "Email verified."),
            );
            setDone(true);
          } catch (e) {
            setNotice(errorMessage(e));
          } finally {
            setBusy(false);
          }
        }}
      >
        <p className="eyebrow">ACCOUNT ACCESS</p>
        <h1>
          {mode === "request"
            ? "A fresh start."
            : mode === "reset"
              ? "Set a new password."
              : "Verify your email."}
        </h1>
        {notice && (
          <p role="status" className="notice">
            {notice}
          </p>
        )}
        {!done && (
          <>
            {mode === "request" && (
              <>
                <p>
                  Local sandbox only: a matching account receives a link in its
                  organization’s administrator sandbox mailbox. No email is
                  sent.
                </p>
                <label>
                  Email address
                  <input
                    name="email"
                    type="email"
                    required
                    autoComplete="email"
                  />
                </label>
              </>
            )}
            {mode === "reset" && (
              <label>
                New password
                <input
                  name="password"
                  type="password"
                  required
                  minLength={12}
                  maxLength={256}
                  autoComplete="new-password"
                />
              </label>
            )}
            {mode === "verify" && (
              <p>Confirm the verification link issued to your account.</p>
            )}
            <button className="primary" disabled={busy}>
              {busy
                ? "Working…"
                : mode === "request"
                  ? "Request sandbox reset link"
                  : mode === "reset"
                    ? "Update password"
                    : "Verify email"}
            </button>
          </>
        )}
        <Link className="text-link" href="/login">
          Return to sign in →
        </Link>
      </form>
    </main>
  );
}
