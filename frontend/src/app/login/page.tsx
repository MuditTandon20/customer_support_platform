"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { login, ApiError } from "@/lib/api-client";
import { saveToken } from "@/lib/auth";

export default function LoginPage() {
  const router = useRouter();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const response = await login({ email, password });
      saveToken(response.token);
      router.push("/tickets");
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.status === 401 ? "Invalid email or password" : err.message);
      } else {
        setError("Something went wrong. Please try again.");
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-[#0B0F14] px-4">
      {/* Ambient grid background — evokes a console/ops-room feel */}
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.04]"
        style={{
          backgroundImage:
            "linear-gradient(#8B98A5 1px, transparent 1px), linear-gradient(90deg, #8B98A5 1px, transparent 1px)",
          backgroundSize: "48px 48px",
        }}
      />

      <div className="relative w-full max-w-[400px] animate-[fadeIn_0.4s_ease-out]">
        {/* Wordmark */}
        <div className="mb-8 flex items-center gap-2.5">
          <span className="h-2 w-2 rounded-full bg-[#3DDC97] shadow-[0_0_8px_#3DDC97]" />
          <span className="font-mono text-xs tracking-[0.2em] text-[#8B98A5] uppercase">
            Helpdesk Console
          </span>
        </div>

        <div className="overflow-hidden rounded-lg border border-[#26303A] bg-[#131920] shadow-2xl">
          {/* Signature: segmented status-color bar */}
          <div className="flex h-1">
            <div className="flex-1 bg-[#F5A623]" />
            <div className="flex-1 bg-[#3DDC97]" />
            <div className="flex-1 bg-[#8B98A5]" />
            <div className="flex-1 bg-[#FF6B6B]" />
          </div>

          <form onSubmit={handleSubmit} className="space-y-5 p-8">
            <div>
              <h1 className="font-display text-2xl font-semibold text-[#E8EDF2]">
                Log in
              </h1>
              <p className="mt-1 text-sm text-[#8B98A5]">
                Access your team&apos;s ticket queue
              </p>
            </div>

            {error && (
              <div className="rounded-md border border-[#FF6B6B]/30 bg-[#FF6B6B]/10 px-3 py-2.5 text-sm text-[#FF6B6B]">
                {error}
              </div>
            )}

            <div className="space-y-1.5">
              <label className="block text-xs font-medium tracking-wide text-[#8B98A5] uppercase">
                Email
              </label>
              <input
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full rounded-md border border-[#26303A] bg-[#1B232C] px-3.5 py-2.5 text-[#E8EDF2] outline-none transition-colors placeholder:text-[#8B98A5]/50 focus:border-[#F5A623]"
                placeholder="you@company.com"
              />
            </div>

            <div className="space-y-1.5">
              <label className="block text-xs font-medium tracking-wide text-[#8B98A5] uppercase">
                Password
              </label>
              <input
                type="password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full rounded-md border border-[#26303A] bg-[#1B232C] px-3.5 py-2.5 text-[#E8EDF2] outline-none transition-colors focus:border-[#F5A623]"
                placeholder="••••••••"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded-md bg-[#F5A623] px-4 py-2.5 font-medium text-[#0B0F14] transition-all hover:bg-[#FFB84D] hover:scale-[1.01] active:scale-[0.99] disabled:opacity-50"
            >
              {loading ? "Logging in..." : "Log in"}
            </button>

            <p className="text-center text-sm text-[#8B98A5]">
              Don&apos;t have an organization yet?{" "}
              <a href="/signup" className="text-[#F5A623] hover:underline">
                Sign up
              </a>
            </p>
          </form>
        </div>
      </div>
    </div>
  );
}