import { apiFetch } from "./api-client";
import type { Ticket } from "@/types";
export interface InboxTicket extends Ticket {
  priority: string;
  labels: string[];
  updated_at: string;
  customer_name: string;
  customer_email: string;
  message_count: number;
}
export interface Message {
  id: string;
  body: string;
  internal: boolean;
  delivery: string;
  created_at: string;
  author_name: string;
  author_role: string;
}
export interface Article {
  id: string;
  title: string;
  content: string;
  visibility: "team" | "public";
  version: number;
  updated_at: string;
  expires_at: string | null;
}
export interface Proposal {
  id: string;
  draft: string;
  citations: { id: string; title: string; excerpt: string; version: number }[];
  abstained: boolean;
  mode: string;
}
export interface Action {
  id: string;
  ticket_id: string;
  action: string;
  arguments: Record<string, string>;
  state: string;
  result: Record<string, unknown>;
  created_at: string;
}
export interface WorkflowNode {
  id: string;
  kind: string;
  value: string;
  next: string | null;
  otherwise: string | null;
}
export interface Workflow {
  id: string;
  name: string;
  draft: { nodes: WorkflowNode[] };
  published: { nodes: WorkflowNode[] } | null;
  version: number;
}
export interface WorkflowRun {
  id: string;
  workflow_id: string;
  ticket_id: string;
  state: string;
  simulation: boolean;
  trace: { node: string; kind: string; value: string }[];
  created_at: string;
}
export interface Analytics {
  totals: {
    total: number;
    backlog: number;
    resolved: number;
    over_24h: number;
    unassigned: number;
  };
  statuses: { status: string; count: number }[];
  daily: { date: string; count: number }[];
  ai: {
    proposals: number;
    abstained: number;
    cited: number;
    model_cost_usd: number;
    mode: string;
  };
  first_response_minutes: number | null;
  csat_average: number | null;
  definitions: Record<string, string>;
}
export interface Integration {
  id: string;
  name: string;
  status: string;
  detail: string;
}
export interface OrganizationSettings {
  name: string;
  slug: string;
  settings: { reply_tone?: string; onboarding_complete?: boolean };
}
export interface Billing {
  plan: string;
  provider: string;
  charges_enabled: boolean;
  usage: Record<string, number>;
  limits: Record<string, number>;
  invoices: unknown[];
}
export interface AuditEvent {
  id: string;
  action: string;
  resource_id: string;
  actor_id: string;
  detail: Record<string, unknown>;
  created_at: string;
}
export const read = <T>(path: string, signal?: AbortSignal) =>
  apiFetch<T>(path, { signal });
export const write = <T>(path: string, body: unknown = {}, method = "POST") =>
  apiFetch<T>(path, { method, body: JSON.stringify(body) });
export const remove = (path: string) => apiFetch(path, { method: "DELETE" });
export const key = () => crypto.randomUUID();
export const errorMessage = (e: unknown) =>
  e instanceof Error ? e.message : "The request failed. Try again.";
