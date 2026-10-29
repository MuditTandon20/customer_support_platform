"use client";

import { useEffect, useState, useMemo } from "react";
import Link from "next/link";
import { listUsers, updateUserRole, getMe, ApiError } from "@/lib/api-client";
import type { UserSummary, Role, User } from "@/types";

const ROLES: Role[] = ["admin", "agent", "customer"];

export default function AdminPage() {
  const [users, setUsers] = useState<UserSummary[]>([]);
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [updatingUserId, setUpdatingUserId] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([listUsers(), getMe()])
      .then(([uList, me]) => {
        setUsers(uList);
        setCurrentUser(me);
      })
      .catch((err) => {
        setError(
          err instanceof ApiError ? err.message : "Failed to load team data",
        );
      })
      .finally(() => setLoading(false));
  }, []);

  async function handleRoleChange(userId: string, role: Role) {
    setUpdatingUserId(userId);
    setError(null);
    setSuccessMessage(null);
    try {
      await updateUserRole(userId, role);
      setUsers((prev) =>
        prev.map((u) => (u.id === userId ? { ...u, role } : u)),
      );
      setSuccessMessage("Role updated successfully");
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Failed to update role");
    } finally {
      setUpdatingUserId(null);
    }
  }

  const roleStats = useMemo(() => {
    const stats = { total: users.length, admin: 0, agent: 0, customer: 0 };
    for (const u of users) {
      if (u.role in stats) {
        stats[u.role]++;
      }
    }
    return stats;
  }, [users]);

  if (loading) {
    return (
      <div className="flex min-h-[400px] flex-col items-center justify-center gap-3">
        <span className="h-3 w-3 animate-ping rounded-full bg-[#3DDC97]" />
        <span className="font-mono text-sm text-[#8B98A5]">
          Loading organization data...
        </span>
      </div>
    );
  }

  return (
    <div className="animate-[fadeIn_0.4s_ease-out] space-y-6">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl font-semibold text-[#E8EDF2]">
            Organization Administration
          </h1>
          <p className="mt-1 text-sm text-[#8B98A5]">
            Manage team members, assign access roles, and view organization
            details.
          </p>
        </div>
        <Link
          href="/tickets"
          className="rounded-md border border-[#26303A] bg-[#131920] px-3.5 py-1.5 font-mono text-xs text-[#8B98A5] transition-colors hover:text-[#E8EDF2]"
        >
          ← Back to Tickets
        </Link>
      </div>

      {/* Notifications */}
      {error && (
        <div className="rounded-md border border-[#FF6B6B]/30 bg-[#FF6B6B]/10 px-4 py-3 text-sm text-[#FF6B6B]">
          {error}
        </div>
      )}
      {successMessage && (
        <div className="rounded-md border border-[#3DDC97]/30 bg-[#3DDC97]/10 px-4 py-3 text-sm text-[#3DDC97]">
          ✓ {successMessage}
        </div>
      )}

      {/* Organization Info Card (Item 5) */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-lg border border-[#26303A] bg-[#131920] p-4">
          <span className="font-mono text-xs text-[#8B98A5]">ORGANIZATION</span>
          <h3 className="mt-1 font-display text-lg font-semibold text-[#E8EDF2]">
            {currentUser?.org_name || "Support Organization"}
          </h3>
          <p className="mt-1 font-mono text-[11px] text-[#8B98A5]">
            Slug: {currentUser?.org_slug || "default"}
          </p>
        </div>

        <div className="rounded-lg border border-[#26303A] bg-[#131920] p-4">
          <span className="font-mono text-xs text-[#8B98A5]">
            TOTAL MEMBERS
          </span>
          <h3 className="mt-1 font-mono text-xl font-bold text-[#E8EDF2]">
            {roleStats.total}
          </h3>
          <p className="mt-1 text-[11px] text-[#8B98A5]">Across all roles</p>
        </div>

        <div className="rounded-lg border border-[#26303A] bg-[#131920] p-4">
          <span className="font-mono text-xs text-[#8B98A5]">
            SUPPORT AGENTS
          </span>
          <h3 className="mt-1 font-mono text-xl font-bold text-[#3DDC97]">
            {roleStats.agent}
          </h3>
          <p className="mt-1 text-[11px] text-[#8B98A5]">
            Triage & ticket resolution
          </p>
        </div>

        <div className="rounded-lg border border-[#26303A] bg-[#131920] p-4">
          <span className="font-mono text-xs text-[#8B98A5]">
            ADMINISTRATORS
          </span>
          <h3 className="mt-1 font-mono text-xl font-bold text-[#F5A623]">
            {roleStats.admin}
          </h3>
          <p className="mt-1 text-[11px] text-[#8B98A5]">
            Full platform access
          </p>
        </div>
      </div>

      {/* Team Members Management Table */}
      <div className="rounded-lg border border-[#26303A] bg-[#131920] shadow-sm">
        <div className="border-b border-[#26303A] px-6 py-4">
          <h2 className="font-display text-base font-semibold text-[#E8EDF2]">
            Team Members & Permissions
          </h2>
          <p className="mt-0.5 text-xs text-[#8B98A5]">
            Change access levels for users in your organization.
          </p>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full min-w-[600px] text-left">
            <thead>
              <tr className="border-b border-[#26303A] bg-[#1B232C]/60 text-xs font-medium tracking-wide text-[#8B98A5] uppercase">
                <th className="px-6 py-3">Member</th>
                <th className="px-6 py-3">Email</th>
                <th className="px-6 py-3">Current Role</th>
                <th className="px-6 py-3 text-right">Change Role</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#26303A]">
              {users.map((u) => {
                const isSelf = u.id === currentUser?.user_id;
                const isUpdating = updatingUserId === u.id;
                return (
                  <tr
                    key={u.id}
                    className="transition-colors hover:bg-[#1B232C]/30"
                  >
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <div className="flex h-8 w-8 items-center justify-center rounded-full bg-[#1B232C] font-mono text-xs font-medium text-[#E8EDF2]">
                          {u.full_name.charAt(0).toUpperCase()}
                        </div>
                        <div>
                          <div className="text-sm font-medium text-[#E8EDF2]">
                            {u.full_name}
                          </div>
                          {isSelf && (
                            <span className="font-mono text-[10px] text-[#F5A623]">
                              (You)
                            </span>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 font-mono text-xs text-[#8B98A5]">
                      {u.email}
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={`inline-block rounded-full px-2.5 py-0.5 font-mono text-xs uppercase ${
                          u.role === "admin"
                            ? "bg-[#F5A623]/10 text-[#F5A623]"
                            : u.role === "agent"
                              ? "bg-[#3DDC97]/10 text-[#3DDC97]"
                              : "bg-[#8B98A5]/10 text-[#8B98A5]"
                        }`}
                      >
                        {u.role}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <select
                        value={u.role}
                        onChange={(e) =>
                          handleRoleChange(u.id, e.target.value as Role)
                        }
                        disabled={isUpdating}
                        className="rounded-md border border-[#26303A] bg-[#1B232C] px-3 py-1.5 font-mono text-xs text-[#E8EDF2] outline-none transition-colors focus:border-[#F5A623] disabled:opacity-50"
                      >
                        {ROLES.map((r) => (
                          <option key={r} value={r}>
                            {r.toUpperCase()}
                          </option>
                        ))}
                      </select>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
