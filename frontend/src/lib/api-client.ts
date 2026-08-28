import { getToken } from "./auth";
import type {
  AuthResponse,
  CreateTicketRequest,
  LoginRequest,
  SignupRequest,
  Ticket,
  TicketCategoryResponse,
  TicketDraftReplyResponse,
  TicketSummaryResponse,
  UpdateTicketRequest,
  User,
} from "@/types";

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://127.0.0.1:3000";

class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

// Central fetch wrapper — every request goes through here, so auth headers
// and error handling live in exactly one place (same principle as AppError in Rust).
async function apiFetch<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const token = getToken();

  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...options.headers,
  };

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const body = await response.json().catch(() => ({ error: "Unknown error" }));
    throw new ApiError(response.status, body.error || "Request failed");
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
  data: UpdateTicketRequest
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
  id: string
): Promise<TicketDraftReplyResponse> {
  return apiFetch<TicketDraftReplyResponse>(`/tickets/${id}/draft-reply`);
}

export { ApiError };