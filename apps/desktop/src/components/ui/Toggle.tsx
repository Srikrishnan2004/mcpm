import type { ReactNode } from "react";
import { Check } from "lucide-react";
import { cn } from "../../lib/cn";

export function Toggle({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative inline-flex h-6 w-10 shrink-0 items-center rounded-full transition-colors duration-150",
        "focus-visible:shadow-focus disabled:cursor-not-allowed disabled:opacity-50",
        checked ? "bg-gray-1000" : "bg-gray-400",
      )}
    >
      <span
        className={cn(
          "absolute left-0.5 size-5 rounded-full bg-bg-100 shadow-[0_1px_2px_#00000029] transition-transform duration-150",
          checked && "translate-x-4",
        )}
      />
    </button>
  );
}

export function Checkbox({
  checked,
  onChange,
  children,
  disabled,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  children: ReactNode;
  disabled?: boolean;
}) {
  return (
    <label className={cn("inline-flex items-center gap-2 label-14", disabled ? "text-gray-700" : "cursor-pointer")}>
      <button
        type="button"
        role="checkbox"
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={cn(
          "flex size-4 items-center justify-center rounded-[4px] transition-colors duration-150 focus-visible:shadow-focus",
          checked ? "bg-gray-1000 text-bg-100" : "bg-bg-100 shadow-[0_0_0_1px_var(--gray-a-600)]",
        )}
      >
        {checked && <Check className="size-3" strokeWidth={3} />}
      </button>
      {children}
    </label>
  );
}
