"use client";
import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ApiError, login, signup } from "@/lib/api-client";
import { saveToken } from "@/lib/auth";
import { errorMessage } from "@/lib/platform";
export default function AuthForm({ register = false }: { register?: boolean }) {
  const [error, setError] = useState(""),
    [existingAccount, setExistingAccount] = useState(false),
    [busy, setBusy] = useState(false);
  const router = useRouter();
  return (
    <main className="workspace auth-page">
      <section className="auth-intro">
        <Link className="brand" href="/">
          <span className="brand-mark">a</span>aster
          <span className="brand-caption">SUPPORT OS</span>
        </Link>
        <div>
          <p className="eyebrow">A MORE CONSIDERED KIND OF SUPPORT</p>
          <h1>
            Good conversations.
            <br />
            Grounded answers.
            <br />
            <span>Human judgment.</span>
          </h1>
          <p>
            Your team, your knowledge, your customers.
            <br />
            One thoughtful place to bring them together.
          </p>
        </div>
        <small>Built for clarity. Designed for people.</small>
      </section>
      <section className="auth-fields">
        <form
          onSubmit={async (e) => {
            e.preventDefault();
            setBusy(true);
            setError("");
            setExistingAccount(false);
            const f = new FormData(e.currentTarget);
            try {
              const email = String(f.get("email")),
                password = String(f.get("password"));
              const result = register
                ? await signup({
                    email,
                    password,
                    organization_name: String(f.get("organization")),
                    full_name: String(f.get("name")),
                  })
                : await login({ email, password });
              saveToken(result.token);
              router.push(
                register ? "/workspace/settings" : "/workspace/dashboard",
              );
            } catch (e) {
              setExistingAccount(
                register && e instanceof ApiError && e.status === 409,
              );
              setError(errorMessage(e));
            } finally {
              setBusy(false);
            }
          }}
        >
          <p className="eyebrow">
            {register ? "YOUR NEXT CHAPTER" : "WELCOME BACK"}
          </p>
          <h2>
            {register
              ? "Make room for better support."
              : "A little clarity starts here."}
          </h2>
          <p>
            {register
              ? "Create a test organization. Use a synthetic .test email address; do not enter real customer data."
              : "Sign in to your support workspace."}
          </p>
          {error && (
            <div className="error-banner" role="alert">
              {error}
              {existingAccount && (
                <p>
                  <Link href="/login">Sign in to your existing account</Link>
                </p>
              )}
            </div>
          )}
          {register && (
            <>
              <label>
                Organization name
                <input
                  required
                  name="organization"
                  autoComplete="organization"
                  maxLength={100}
                  placeholder="Your company"
                />
              </label>
              <label>
                Full name
                <input
                  required
                  name="name"
                  autoComplete="name"
                  maxLength={200}
                  placeholder="Your name"
                />
              </label>
            </>
          )}
          <label>
            Email address
            <input
              type="email"
              required
              name="email"
              autoComplete="email"
              maxLength={254}
              placeholder="tester@example.test"
            />
          </label>
          <label>
            Password
            <input
              required
              name="password"
              type="password"
              autoComplete={register ? "new-password" : "current-password"}
              minLength={register ? 12 : 1}
              maxLength={256}
              placeholder={
                register ? "At least 12 characters" : "Your password"
              }
            />
          </label>
          <button disabled={busy} className="primary full">
            {busy
              ? "Opening your workspace…"
              : register
                ? "Create workspace ↗"
                : "Sign in →"}
          </button>
          <p className="auth-switch">
            {register ? "Already have a workspace?" : "New to Aster?"}{" "}
            <Link href={register ? "/login" : "/signup"}>
              {register ? "Sign in" : "Create an organization"}
            </Link>
          </p>
          {!register && (
            <p className="auth-switch">
              Have an invitation? <Link href="/join">Join your team</Link>
            </p>
          )}
          {!register && (
            <Link className="text-link" href="/forgot-password">
              Reset your password
            </Link>
          )}
          <div className="safe-note">
            {register
              ? "No payment provider enabled. No charges."
              : "Your organization controls membership. Joining a team requires an invitation."}
          </div>
        </form>
      </section>
    </main>
  );
}
