import type { TicketStatus } from "@/types";

const STATUS_STYLES: Record<
  TicketStatus,
  { bg: string; text: string; label: string }
> = {
  new: { bg: "bg-[#F5A623]/10", text: "text-[#F5A623]", label: "New" },
  open: { bg: "bg-[#F5A623]/10", text: "text-[#F5A623]", label: "Open" },
  pending: { bg: "bg-[#8B98A5]/10", text: "text-[#8B98A5]", label: "Pending" },
  on_hold: { bg: "bg-[#8B98A5]/10", text: "text-[#8B98A5]", label: "On Hold" },
  solved: { bg: "bg-[#3DDC97]/10", text: "text-[#3DDC97]", label: "Solved" },
  closed: { bg: "bg-[#3DDC97]/10", text: "text-[#3DDC97]", label: "Closed" },
};

export function StatusBadge({ status }: { status: TicketStatus }) {
  const style = STATUS_STYLES[status];
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-1 font-mono text-xs font-medium ${style.bg} ${style.text}`}
    >
      {style.label}
    </span>
  );
}
