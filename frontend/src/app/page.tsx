import type { Metadata } from "next";
import Link from "next/link";
export const metadata: Metadata = {
  title: "Aster — Support with a little more thought",
  description:
    "An evidence-first customer support workspace with human control.",
};
export default function Home() {
  return (
    <main className="workspace marketing">
      <nav className="marketing-nav">
        <Link prefetch={false} className="brand" href="/">
          <span className="brand-mark">a</span>aster
          <span className="brand-caption">SUPPORT OS</span>
        </Link>
        <div>
          <Link prefetch={false} href="#principles">The approach</Link>
          <Link prefetch={false} href="/login">Sign in</Link>
          <Link prefetch={false} className="button primary" href="/signup">
            Create workspace ↗
          </Link>
        </div>
      </nav>
      <section className="hero">
        <span className="hero-pill">
          <span className="online-dot" /> SUPPORT, WITH HUMAN JUDGMENT
        </span>
        <h1>
          Less noise.
          <br />
          More <em>understanding.</em>
        </h1>
        <p>
          A considered workspace for customer support.
          <br />
          Bring conversations, knowledge, and controlled automation
          <br />
          together. Keep your team in charge.
        </p>
        <div className="hero-actions">
          <Link prefetch={false} className="button primary" href="/signup">
            Build your workspace ↗
          </Link>
          <Link prefetch={false} className="button secondary" href="/login">
            Open your console →
          </Link>
        </div>
        <small>Local-first setup · No payment provider enabled</small>
        <div className="hero-product depth-hero">
          <div className="preview-sidebar">
            <span className="brand-mark">a</span>
            <span>Overview</span>
            <span className="selected">Conversations</span>
            <span>Knowledge</span>
            <span>Workflows</span>
          </div>
          <div className="automation-float float-evidence" aria-hidden="true">
            Evidence ? review
          </div>
          <div className="automation-float float-control" aria-hidden="true">
            Human control
          </div>
          <div className="preview-main">
            <p className="eyebrow">A CALMER WAY TO HELP</p>
            <h2>Every answer has a starting point.</h2>
            <div className="preview-message">
              <span className="avatar">C</span>
              <div>
                <strong>A customer needs a hand</strong>
                <p>Your team sees the full conversation.</p>
              </div>
            </div>
            <div className="preview-evidence">
              <span>✧</span>
              <div>
                <strong>Find the evidence. Make it your own.</strong>
                <p>Knowledge sources → Cited draft → Human review → Send</p>
              </div>
            </div>
            <div className="preview-status">
              <span className="online-dot" /> Illustrative product flow — not
              live workspace metrics
            </div>
          </div>
        </div>
      </section>
      <section className="principles" id="principles">
        <p className="eyebrow">GOOD SUPPORT IS A PRACTICE</p>
        <h2>
          Thoughtful by design.
          <br />
          Accountable by default.
        </h2>
        <div className="article-grid">
          {[
            [
              "01",
              "Evidence before answers",
              "Current, tenant-scoped knowledge and inspectable citations. A clear abstention when the answer is missing.",
            ],
            [
              "02",
              "Control before impact",
              "Allowlisted local actions. Explicit human review for sensitive requests. No silent external execution.",
            ],
            [
              "03",
              "Clarity before claims",
              "Metrics from actual conversations. Visible limits and honest integration states. No invented savings.",
            ],
          ].map(([n, t, d]) => (
            <article className="panel form-panel" key={n}>
              <span className="eyebrow">{n}</span>
              <h3>{t}</h3>
              <p>{d}</p>
            </article>
          ))}
        </div>
      </section>
      <footer className="marketing-footer">
        <span>aster · Support with a little more thought.</span>
        <Link prefetch={false} href="/signup">Start your workspace ↗</Link>
      </footer>
    </main>
  );
}
