const TOKEN_KEY = "auth_token";

export function saveToken(token: string): void {
  // The actual session is an HttpOnly API cookie; never persist the bearer secret.
  void token;
  localStorage.setItem(TOKEN_KEY, "cookie-session");
}

export function getToken(): string | null {
  if (typeof window === "undefined") return null; // server-side render safety
  return localStorage.getItem(TOKEN_KEY);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}
