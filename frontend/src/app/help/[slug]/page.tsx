"use client";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useState } from "react";
import { useResource, State, Heading } from "@/components/WorkspaceUI";
import type { Article } from "@/lib/platform";
export default function Help() {
  const { slug } = useParams<{ slug: string }>();
  const r = useResource<Article[]>(`/public/help/${encodeURIComponent(slug)}`),
    [query, setQuery] = useState("");
  return (
    <main className="workspace public-page">
      <Link href="/" className="brand">
        <span className="brand-mark">a</span>aster help
      </Link>
      <Heading
        eyebrow="HELP CENTER"
        title="A good place to find your answer."
        description="Current, published knowledge from your support team."
      >
        <Link className="button primary" href="/portal">
          Contact support ↗
        </Link>
      </Heading>
      <label className="help-search">
        Search help articles
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="What can we help with?"
        />
      </label>
      <State loading={r.loading} error={r.error} empty={!r.data?.length}>
        {r.data?.length ? (
          <div className="stack">
            {r.data
              .filter((a) =>
                `${a.title} ${a.content}`
                  .toLowerCase()
                  .includes(query.toLowerCase()),
              )
              .map((a) => (
                <details className="panel help-article" key={a.id}>
                  <summary>{a.title}</summary>
                  <div className="prose-text">{a.content}</div>
                </details>
              ))}
          </div>
        ) : (
          <>
            No public articles are available yet. Contact your support team for
            help.
          </>
        )}
      </State>
    </main>
  );
}
