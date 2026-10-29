import { getToken } from "./auth";
import type {
  AuthResponse,
  CreateTicketRequest,
  LoginRequest,
  Role,
  SignupRequest,
  Ticket,
  TicketCategoryResponse,
  TicketDraftReplyResponse,
  TicketSummaryResponse,
  UpdateTicketRequest,
  User,
  UserSummary,
} from "@/types";

const API_BASE_URL = "/api";

class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

// Central fetch wrapper — every request goes through here, so auth headers
// and error handling live in exactly one place (same principle as AppError in Rust).
export async function apiFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const token = getToken();

  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...(token && token !== "cookie-session"
      ? { Authorization: `Bearer ${token}` }
      : {}),
    ...options.headers,
  };

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    credentials: "include",
    headers,
  });

  if (!response.ok) {
    const raw = await response.text();
    let message = raw || "Request failed";
    try {
      message = JSON.parse(raw).error || message;
    } catch {}
    throw new ApiError(response.status, message);
  }

  return response.json() as Promise<T>;
}

// ---- Auth ----

export function signup(data: SignupRequest): Promise<AuthResponse> {
  return apiFetch<AuthResponse>("/auth/signup", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function login(data: LoginRequest): Promise<AuthResponse> {
  return apiFetch<AuthResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function getMe(): Promise<User> {
  return apiFetch<User>("/me");
}

// ---- Tickets ----

export function listTickets(): Promise<Ticket[]> {
  return apiFetch<Ticket[]>("/tickets");
}

export function getTicket(id: string): Promise<Ticket> {
  return apiFetch<Ticket>(`/tickets/${id}`);
}

export function createTicket(data: CreateTicketRequest): Promise<Ticket> {
  return apiFetch<Ticket>("/tickets", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function updateTicket(
  id: string,
  data: UpdateTicketRequest,
): Promise<Ticket> {
  return apiFetch<Ticket>(`/tickets/${id}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
}

// ---- AI Copilot ----

export function getTicketSummary(id: string): Promise<TicketSummaryResponse> {
  return apiFetch<TicketSummaryResponse>(`/tickets/${id}/summary`);
}

export function getTicketCategory(id: string): Promise<TicketCategoryResponse> {
  return apiFetch<TicketCategoryResponse>(`/tickets/${id}/category`);
}

export function getTicketDraftReply(
  id: string,
): Promise<TicketDraftReplyResponse> {
  return apiFetch<TicketDraftReplyResponse>(`/tickets/${id}/draft-reply`);
}

export { ApiError };

export function listUsers(): Promise<UserSummary[]> {
  return apiFetch<UserSummary[]>("/users");
}

export function updateUserRole(userId: string, role: Role): Promise<unknown> {
  return apiFetch(`/users/${userId}/role`, {
    method: "PATCH",
    body: JSON.stringify({ role }),
  });
}
