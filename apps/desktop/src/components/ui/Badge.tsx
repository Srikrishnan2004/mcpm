import type { ReactNode } from "react";
import { cn } from "../../lib/cn";

type Tone = "gray" | "blue" | "green" | "amber" | "red";

const tones: Record<Tone, string> = {
  gray: "bg-gray-200 text-gray-1000",
  blue: "bg-blue-200 text-blue-900",
  green: "bg-green-200 text-green-900",
  amber: "bg-amber-200 text-amber-900",
  red: "bg-red-200 text-red-900",
};

export function Badge({
  tone = "gray",
  size = "sm",
  icon,
  className,
  children,
}: {
  tone?: Tone;
  size?: "sm" | "md";
  icon?: ReactNode;
  className?: string;
  children: ReactNode;
}) {
  return (
    <span
      className={cn(
        "inline-flex shrink-0 items-center gap-1 rounded-full font-medium tabular-nums whitespace-nowrap",
        size === "sm" ? "h-5 px-2 text-[11px]" : "h-6 px-2.5 text-xs",
        tones[tone],
        className,
      )}
    >
      {icon}
      {children}
    </span>
  );
}
