"use client";
import { useRef, useState } from "react";
import { useUser } from "./WorkspaceShell";
import { useResource, Heading, State, Notice, Pill, date } from "./WorkspaceUI";
import {
  read,
  write,
  key,
  errorMessage,
  type Workflow,
  type WorkflowNode,
  type WorkflowRun,
  type InboxTicket,
} from "@/lib/platform";
const template: WorkflowNode[] = [
  {
    id: "start",
    kind: "trigger",
    value: "manual",
    next: "tag",
    otherwise: null,
  },
  {
    id: "tag",
    kind: "action",
    value: "label:reviewed",
    next: "handoff",
    otherwise: null,
  },
  {
    id: "handoff",
    kind: "approval",
    value: "Human review",
    next: null,
    otherwise: null,
  },
];
export function WorkflowStudio() {
  const r = useResource<Workflow[]>("/workflows"),
    history = useResource<WorkflowRun[]>("/workflow-runs"),
    tickets = useResource<InboxTicket[]>("/inbox");
  const [nodes, setNodes] = useState<WorkflowNode[]>(template),
    [name, setName] = useState("A considered handoff"),
    [id, setId] = useState(""),
    [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false),
    [ticket, setTicket] = useState(""),
    [zoom, setZoom] = useState(1),
    [trace, setTrace] = useState<WorkflowRun | null>(null),
    [selected, setSelected] = useState("tag");
  const dragged = useRef<number | null>(null);
  const user = useUser();
  const admin = user?.role === "admin";
  function chain(list: WorkflowNode[]) {
    setNodes(
      list.map((n, i) => ({
        ...n,
        next: list[i + 1]?.id || null,
        otherwise: null,
      })),
    );
  }
  function move(index: number, delta: number) {
    const target = index + delta;
    if (index === 0 || target < 1 || target >= nodes.length) return;
    const list = [...nodes];
    [list[index], list[target]] = [list[target], list[index]];
    chain(list);
  }
  function edit(field: keyof WorkflowNode, value: string | null) {
    setNodes((ns) =>
      ns.map((n) => (n.id === selected ? { ...n, [field]: value } : n)),
    );
  }
  async function save() {
    setBusy(true);
    try {
      const saved = await write<Workflow>(
        id ? `/workflows/${id}` : "/workflows",
        { name, definition: { nodes } },
        id ? "PATCH" : "POST",
      );
      setId(saved.id);
      setNotice("Draft saved. Publish it before running against a ticket.");
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function publish() {
    if (!id) return;
    setBusy(true);
    try {
      await write(`/workflows/${id}`, { name, definition: { nodes } }, "PATCH");
      const result = await write<{ version: number }>(
        `/workflows/${id}/publish`,
      );
      setNotice(
        `Version ${result.version} published. Execution uses this saved version.`,
      );
      r.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  async function run(simulation: boolean) {
    if (!id || !ticket) return;
    setBusy(true);
    try {
      if (simulation && admin)
        await write(
          `/workflows/${id}`,
          { name, definition: { nodes } },
          "PATCH",
        );
      const result = await write<WorkflowRun>(`/workflows/${id}/run`, {
        ticket_id: ticket,
        simulation,
        idempotency_key: key(),
      });
      setTrace(result);
      setNotice(
        `${simulation ? "Simulation" : "Run"} ${result.state}. ${simulation ? "No ticket changes made." : ""}`,
      );
      history.refresh();
    } catch (e) {
      setNotice(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }
  const node = nodes.find((n) => n.id === selected);
  return (
    <>
      <Heading
        eyebrow="WORKFLOW STUDIO"
        title="Automate the routine. Keep the judgment."
        description="Bounded, server-executed workflows with versioned publishing and a dry run before action."
      >
        <button disabled={busy || !admin} onClick={save}>
          Save draft
        </button>
        <button
          className="primary"
          disabled={busy || !id || !admin}
          onClick={publish}
        >
          Publish version ↗
        </button>
      </Heading>
      <Notice text={notice} />
      <State loading={r.loading} error={r.error}>
        <div className="studio-toolbar">
          <label>
            Workflow
            <select
              value={id}
              onChange={(e) => {
                const w = r.data?.find((w) => w.id === e.target.value);
                setId(w?.id || "");
                setNodes(w?.draft.nodes || template);
                setName(w?.name || "Untitled workflow");
                setSelected(w?.draft.nodes[0]?.id || "start");
              }}
            >
              <option value="">New workflow</option>
              {r.data?.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name} · v{w.version}
                </option>
              ))}
            </select>
          </label>
          <label>
            Name
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              maxLength={160}
            />
          </label>
          <Pill tone="violet">{id ? "Saved draft" : "Unsaved draft"}</Pill>
        </div>
      </State>
      <div className="studio-layout">
        <section className="workflow-canvas" aria-label="Workflow graph">
          <div className="canvas-caption">
            <span className="online-dot" /> MANUAL TRIGGER · MAX 32 STEPS
          </div>
          <div
            className="workflow-node-list"
            style={{
              transform: `scale(${zoom})`,
              transformOrigin: "top center",
            }}
          >
            {nodes.map((n, i) => (
              <div key={n.id} className="workflow-node-wrap">
                <button
                  className={`workflow-node ${selected === n.id ? "selected" : ""}`}
                  onClick={() => setSelected(n.id)}
                  draggable={admin && i > 0}
                  onDragStart={() => {
                    dragged.current = i;
                  }}
                  onDragOver={(e) => e.preventDefault()}
                  onDrop={(e) => {
                    e.preventDefault();
                    if (
                      dragged.current !== null &&
                      i > 0 &&
                      dragged.current > 0
                    ) {
                      const list = [...nodes];
                      const [moved] = list.splice(dragged.current, 1);
                      list.splice(i, 0, moved);
                      chain(list);
                    }
                    dragged.current = null;
                  }}
                >
                  <span className={`node-symbol ${n.kind}`}>
                    {n.kind === "trigger"
                      ? "ϟ"
                      : n.kind === "action"
                        ? "⌘"
                        : n.kind === "approval"
                          ? "◇"
                          : "⑂"}
                  </span>
                  <span>
                    <small>{n.kind.toUpperCase()}</small>
                    <strong>{n.value || n.id}</strong>
                    <small className="node-edge">
                      {n.next ? `Next → ${n.next}` : "End of path"}
                      {n.otherwise ? ` · Else → ${n.otherwise}` : ""}
                    </small>
                  </span>
                  <span className="node-id">{n.id}</span>
                </button>
                {i < nodes.length - 1 && (
                  <div className="workflow-edge" aria-hidden="true">
                    ↓
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="canvas-tools">
            <button
              aria-label="Zoom out"
              onClick={() => setZoom((z) => Math.max(0.5, z - 0.1))}
            >
              −
            </button>
            <span>{Math.round(zoom * 100)}%</span>
            <button
              aria-label="Zoom in"
              onClick={() => setZoom((z) => Math.min(1.4, z + 0.1))}
            >
              ＋
            </button>
            <button onClick={() => setZoom(1)}>Reset</button>
          </div>
        </section>
        <section className="panel node-inspector">
          <div className="panel-heading">
            <h2>Step inspector</h2>
            <Pill>{nodes.length} / 32</Pill>
          </div>
          {node && (
            <div className="form-panel">
              <label>
                Node ID
                <input value={node.id} readOnly />
              </label>
              <label>
                Step type
                <select
                  disabled={node.kind === "trigger" || !admin}
                  value={node.kind}
                  onChange={(e) => edit("kind", e.target.value)}
                >
                  {[
                    "trigger",
                    "condition",
                    "action",
                    "approval",
                    "fallback",
                  ].map((k) => (
                    <option key={k}>{k}</option>
                  ))}
                </select>
              </label>
              <label>
                Value
                <input
                  disabled={!admin}
                  value={node.value}
                  onChange={(e) => edit("value", e.target.value)}
                />
              </label>
              <p className="muted">
                Actions: label:reviewed or status:open. Conditions compare
                ticket priority. Approval pauses for a manual handoff; it does
                not auto-resume.
              </p>
              <label>
                Next node
                <select
                  value={node.next || ""}
                  onChange={(e) => edit("next", e.target.value || null)}
                  disabled={!admin}
                >
                  <option value="">End</option>
                  {nodes
                    .filter((n) => n.id !== node.id)
                    .map((n) => (
                      <option key={n.id}>{n.id}</option>
                    ))}
                </select>
              </label>
              {node.kind === "condition" && (
                <label>
                  Otherwise
                  <select
                    disabled={!admin}
                    value={node.otherwise || ""}
                    onChange={(e) => edit("otherwise", e.target.value || null)}
                  >
                    <option value="">Select a branch</option>
                    {nodes
                      .filter((n) => n.id !== node.id)
                      .map((n) => (
                        <option key={n.id}>{n.id}</option>
                      ))}
                  </select>
                </label>
              )}
              <div className="row">
                <button
                  disabled={!admin || nodes.indexOf(node) <= 1}
                  onClick={() => move(nodes.indexOf(node), -1)}
                >
                  ↑ Move
                </button>
                <button
                  disabled={
                    !admin ||
                    nodes.indexOf(node) === 0 ||
                    nodes.indexOf(node) === nodes.length - 1
                  }
                  onClick={() => move(nodes.indexOf(node), 1)}
                >
                  ↓ Move
                </button>
              </div>
              {node.kind !== "trigger" && (
                <button
                  className="danger"
                  disabled={!admin}
                  onClick={() => {
                    chain(nodes.filter((n) => n.id !== node.id));
                    setSelected("start");
                  }}
                >
                  Remove step
                </button>
              )}
            </div>
          )}
          <div className="form-panel inspector-footer">
            <button
              disabled={!admin || nodes.length >= 32}
              onClick={() => {
                const id = `step-${crypto.randomUUID().slice(0, 6)}`;
                chain([
                  ...nodes,
                  {
                    id,
                    kind: "action",
                    value: "label:reviewed",
                    next: null,
                    otherwise: null,
                  },
                ]);
                setSelected(id);
              }}
            >
              ＋ Add step
            </button>
            <small>
              Reordering reconnects a linear path. Edit branch edges afterward.
            </small>
          </div>
        </section>
      </div>
      {id && (
        <section className="panel form-panel">
          <h2>Version history & draft rollback</h2>
          <p>
            Restore an earlier definition into the draft, inspect it, then
            publish a new version. The published workflow stays unchanged until
            you publish.
          </p>
          <form
            className="row"
            onSubmit={async (e) => {
              e.preventDefault();
              setBusy(true);
              const f = new FormData(e.currentTarget);
              try {
                const result = await write<{
                  draft: { nodes: WorkflowNode[] };
                }>(`/workflows/${id}/rollback`, {
                  version: Number(f.get("version")),
                });
                setNodes(result.draft.nodes);
                setNotice(
                  "Historical version restored into the draft. Publish explicitly to activate it.",
                );
                r.refresh();
              } catch (e) {
                setNotice(errorMessage(e));
              } finally {
                setBusy(false);
              }
            }}
          >
            <label>
              Version number
              <input
                name="version"
                type="number"
                min={1}
                required
                defaultValue={1}
              />
            </label>
            <button disabled={!admin || busy}>Restore draft</button>
            <button
              type="button"
              onClick={async () => {
                try {
                  const versions = await read<
                    { version: number; created_at: string }[]
                  >(`/workflows/${id}/versions`);
                  setNotice(
                    versions.length
                      ? versions
                          .map((v) => `v${v.version} ? ${date(v.created_at)}`)
                          .join(" | ")
                      : "No published versions yet.",
                  );
                } catch (e) {
                  setNotice(errorMessage(e));
                }
              }}
            >
              List published versions
            </button>
          </form>
        </section>
      )}
      <section className="panel simulation-bar">
        <div>
          <h2>See it before you run it.</h2>
          <p>
            Simulation validates the draft without changing the conversation.
          </p>
        </div>
        <label>
          <span className="sr-only">Conversation to test</span>
          <select value={ticket} onChange={(e) => setTicket(e.target.value)}>
            <option value="">Choose a conversation</option>
            {tickets.data?.map((t) => (
              <option key={t.id} value={t.id}>
                {t.subject}
              </option>
            ))}
          </select>
        </label>
        <button disabled={busy || !id || !ticket} onClick={() => run(true)}>
          ▷ Simulate
        </button>
        <button
          className="primary"
          disabled={busy || !id || !ticket || !admin}
          onClick={() => run(false)}
        >
          Run published
        </button>
      </section>
      {trace && (
        <section className="panel form-panel">
          <h2>
            {trace.simulation ? "Simulation" : "Execution"} trace ·{" "}
            {trace.state}
          </h2>
          <ol className="trace-list">
            {trace.trace.map((step, i) => (
              <li key={`${step.node}-${i}`}>
                <Pill>{step.kind}</Pill>
                <strong>{step.node}</strong>
                <span>{step.value}</span>
              </li>
            ))}
          </ol>
        </section>
      )}
      <section className="panel">
        <div className="panel-heading">
          <div>
            <h2>Execution history</h2>
            <p>Inspect the actual path, version and handoff state.</p>
          </div>
          <button onClick={history.refresh}>Refresh</button>
        </div>
        <State
          loading={history.loading}
          error={history.error}
          empty={!history.data?.length}
        >
          {history.data?.length ? (
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>Run</th>
                    <th>Mode</th>
                    <th>State</th>
                    <th>Date</th>
                    <th>Trace</th>
                  </tr>
                </thead>
                <tbody>
                  {history.data.map((run) => (
                    <tr key={run.id}>
                      <td className="mono">{run.id.slice(0, 8)}</td>
                      <td>{run.simulation ? "Simulation" : "Published"}</td>
                      <td>
                        <Pill>{run.state}</Pill>
                      </td>
                      <td>{date(run.created_at)}</td>
                      <td>
                        <button onClick={() => setTrace(run)}>Inspect →</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <>Run a simulation to create your first trace.</>
          )}
        </State>
      </section>
    </>
  );
}
