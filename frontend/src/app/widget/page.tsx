"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import { getToken } from "@/lib/auth";
export default function Widget() {
  const [signedIn, setSignedIn] = useState(false);
  useEffect(() => {
    const timer = setTimeout(() => setSignedIn(!!getToken()), 0);
    return () => clearTimeout(timer);
  }, []);
  return (
    <main className="workspace widget">
      <div className="widget-header">
        <span className="brand-mark">a</span>
        <p className="eyebrow">HERE WHEN YOU NEED US</p>
        <h1>Let&apos;s work through it.</h1>
        <p>Talk to your team in a secure conversation.</p>
      </div>
      <section className="panel form-panel">
        <h2>Your support portal</h2>
        <p>
          Sign in using the email associated with your invitation. Your
          conversation history stays private.
        </p>
        <Link
          className="button primary"
          href={signedIn ? "/workspace/inbox" : "/login"}
          target="_blank"
          rel="noopener"
        >
          {signedIn ? "Open your conversations" : "Sign in to contact support"}{" "}
          ↗
        </Link>
        <small>Opens in a new tab. No anonymous messages or tracking.</small>
      </section>
      <p className="widget-footer">Powered by Aster · Secure customer portal</p>
    </main>
  );
}
