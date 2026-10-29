"use client";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { apiFetch, getMe, ApiError } from "@/lib/api-client";
import { getToken, clearToken } from "@/lib/auth";
import Notifications from "./Notifications";
import type { User } from "@/types";

const UserContext = createContext<User | null>(null);
export const useUser = () => useContext(UserContext);
export const navigation = [
  ["dashboard", "Overview", "◈"],
  ["inbox", "Inbox", "▤"],
  ["customers", "Customers", "◎"],
  ["knowledge", "Knowledge", "▧"],
  ["workflows", "Workflows", "⌘"],
  ["approvals", "Approvals", "◇"],
  ["analytics", "Analytics", "▥"],
  ["integrations", "Integrations", "⊞"],
  ["billing", "Billing", "▣"],
  ["settings", "Settings", "⚙"],
  ["audit", "Audit trail", "≡"],
];
export default function WorkspaceShell({
  children,
}: {
  children: React.ReactNode;
}) {
  const [user, setUser] = useState<User | null>(null),
    [error, setError] = useState(""),
    [light, setLight] = useState(false),
    [menu, setMenu] = useState(false),
    [query, setQuery] = useState("");
  const router = useRouter(),
    path = usePathname(),
    dialog = useRef<HTMLDialogElement>(null),
    search = useRef<HTMLInputElement>(null);
  useEffect(() => {
    let active = true;
    if (!getToken()) {
      router.replace("/login");
      return;
    }
    getMe()
      .then((u) => {
        if (active) {
          setUser(u);
          setLight(localStorage.getItem("aster-theme") === "light");
        }
      })
      .catch((e) => {
        if (!active) return;
        if (e instanceof ApiError && e.status === 401) {
          clearToken();
          router.replace("/login");
        } else
          setError(
            "Cannot reach the API. Check that the backend is running, then retry.",
          );
      });
    return () => {
      active = false;
    };
  }, [router]);
  useEffect(() => {
    const handle = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        dialog.current?.showModal();
        search.current?.focus();
      }
    };
    document.addEventListener("keydown", handle);
    return () => document.removeEventListener("keydown", handle);
  }, []);
  const entries =
    user?.role === "customer"
      ? [
          ["inbox", "My conversations", "▤"],
          ["knowledge", "Help center", "▧"],
        ]
      : navigation.filter(
          ([id]) =>
            user?.role === "admin" ||
            !["billing", "settings", "audit"].includes(id),
        );
  if (error)
    return (
      <main className="state-screen">
        <h1>Connection unavailable</h1>
        <p role="alert">{error}</p>
        <button onClick={() => location.reload()}>Retry connection</button>
      </main>
    );
  if (!user)
    return (
      <main className="state-screen" aria-busy="true">
        <div className="skeleton" />
        <p>Opening your workspace…</p>
      </main>
    );
  const current =
    entries.find(([id]) => path.includes(id))?.[1] ||
    (path.startsWith("/tickets") ? "Conversation" : "Workspace");
  return (
    <UserContext.Provider value={user}>
      <div className="workspace" data-theme={light ? "light" : "dark"}>
        <a className="skip-link" href="#workspace-content">
          Skip to content
        </a>
        <aside className={`sidebar ${menu ? "is-open" : ""}`}>
          <Link href="/workspace/dashboard" className="brand">
            <span className="brand-mark">a</span> aster
            <span className="brand-caption">SUPPORT OS</span>
          </Link>
          <div className="org-switch">
            <span className="avatar">{user.org_name?.slice(0, 1) || "W"}</span>
            <div>
              <strong>{user.org_name}</strong>
              <small>{user.role} workspace</small>
            </div>
            <span className="online-dot" />
          </div>
          <p className="nav-caption">WORKSPACE</p>
          <nav aria-label="Workspace">
            {entries.map(([id, label, icon]) => (
              <Link
                key={id}
                className={
                  path === `/workspace/${id}` ||
                  (id === "inbox" && path.startsWith("/tickets"))
                    ? "nav-item active"
                    : "nav-item"
                }
                href={`/workspace/${id}`}
                onClick={() => setMenu(false)}
              >
                <span aria-hidden="true">{icon}</span>
                {label}
                {id === "approvals" && (
                  <span className="nav-label">Review</span>
                )}
              </Link>
            ))}
          </nav>
          <div className="sidebar-bottom">
            <div className="local-card">
              <span className="online-dot" />
              <strong>Human control. Always.</strong>
              <p>
                Evidence before answers.
                <br />
                Approval before sensitive actions.
              </p>
            </div>
            <button
              className="nav-item"
              onClick={() => {
                const next = !light;
                setLight(next);
                localStorage.setItem("aster-theme", next ? "light" : "dark");
              }}
            >
              {light ? "☾ Dark appearance" : "☼ Light appearance"}
            </button>
            <button
              className="nav-item"
              onClick={async () => {
                try {
                  await apiFetch("/auth/logout", { method: "POST" });
                } finally {
                  clearToken();
                  router.replace("/login");
                }
              }}
            >
              ↪ Sign out
            </button>
          </div>
        </aside>
        <div className="workspace-main">
          <header className="topbar">
            <button
              className="mobile-toggle"
              aria-label="Toggle navigation"
              aria-expanded={menu}
              onClick={() => setMenu((v) => !v)}
            >
              ☰
            </button>
            <div className="breadcrumb">
              Workspace <span>/</span> <strong>{current}</strong>
            </div>
            <div className="topbar-actions">
              <button
                className="command-trigger"
                onClick={() => {
                  dialog.current?.showModal();
                  search.current?.focus();
                }}
              >
                Search workspace <kbd>Ctrl K</kbd>
              </button>
              <span className="mode-pill">LOCAL WORKSPACE</span>
              {user.role !== "customer" && <Notifications />}
              <span className="avatar small">
                {user.role.slice(0, 1).toUpperCase()}
              </span>
            </div>
          </header>
          <main id="workspace-content" className="workspace-content">
            {children}
          </main>
          <footer className="workspace-footer">
            Aster Support OS <span>Tenant isolated · Human reviewed</span>
          </footer>
        </div>
        <dialog
          ref={dialog}
          className="command-dialog"
          onClick={(e) => {
            if (e.target === e.currentTarget) dialog.current?.close();
          }}
        >
          <div className="command-inner">
            <div className="row">
              <label htmlFor="command-query">Go to a page</label>
              <button
                aria-label="Close command palette"
                onClick={() => dialog.current?.close()}
              >
                Esc
              </button>
            </div>
            <input
              ref={search}
              id="command-query"
              placeholder="Type a page name…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            <nav aria-label="Page search">
              {entries
                .filter(([, label]) =>
                  label.toLowerCase().includes(query.toLowerCase()),
                )
                .map(([id, label, icon]) => (
                  <Link
                    key={id}
                    href={`/workspace/${id}`}
                    onClick={() => dialog.current?.close()}
                  >
                    {icon} {label}
                    <span>↵</span>
                  </Link>
                ))}
            </nav>
          </div>
        </dialog>
      </div>
    </UserContext.Provider>
  );
}
