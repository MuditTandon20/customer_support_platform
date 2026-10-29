// ---- Enums (mirror Rust's Role and TicketStatus) ----

export type Role = "admin" | "agent" | "customer";

export type TicketStatus =
  | "new"
  | "open"
  | "pending"
  | "on_hold"
  | "solved"
  | "closed";

// ---- Core entities ----

export interface Ticket {
  id: string;
  organization_id: string;
  created_by: string;
  assigned_to: string | null;
  subject: string;
  description: string | null;
  status: TicketStatus;
  created_at: string; // ISO date string — JSON has no native Date type
}

export interface User {
  user_id: string;
  org_id: string;
  org_name?: string;
  org_slug?: string;
  role: Role;
}


export interface UserSummary {
  id: string;
  email: string;
  full_name: string;
  role: Role;
}


// ---- Auth request/response shapes (mirror SignupRequest, LoginRequest, AuthResponse) ----

export interface SignupRequest {
  organization_name?: string;
  organization_id?: string;
  email: string;
  password: string;
  full_name: string;
  role?: Role;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  user_id: string;
  org_id: string;
}

// ---- Ticket request shapes ----

export interface CreateTicketRequest {
  subject: string;
  description?: string;
}

export interface UpdateTicketRequest {
  unassign?: boolean;
  status?: TicketStatus;
  assigned_to?: string;
}

// ---- AI Copilot response shapes ----

export interface TicketSummaryResponse {
  ticket_id: string | null;
  summary: string;
  key_points: string[];
  detected_sentiment: string | null;
}

export interface TicketCategoryResponse {
  ticket_id: string | null;
  category: string;
  confidence: number;
  priority: string;
  tags: string[];
  reasoning: string | null;
}

export interface TicketDraftReplyResponse {
  ticket_id: string | null;
  draft_reply: string;
  suggested_action: string | null;
  tone_used: string;
}