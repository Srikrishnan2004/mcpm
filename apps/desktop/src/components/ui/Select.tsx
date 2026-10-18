import { forwardRef, type SelectHTMLAttributes } from "react";
import { ChevronDown } from "lucide-react";
import { cn } from "../../lib/cn";

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  size_?: "sm" | "md";
  mono?: boolean;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select(
  { className, size_ = "md", mono, children, ...props },
  ref,
) {
  return (
    <div className={cn("relative w-full", className)}>
      <select
        ref={ref}
        className={cn(
          "w-full cursor-pointer appearance-none rounded-md bg-bg-100 pr-9 pl-3 text-gray-1000 shadow-border outline-none",
          "transition-shadow duration-150 hover:shadow-border-hover focus-visible:shadow-input-focus",
          "disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-700",
          size_ === "sm" ? "h-8 text-sm" : "h-9 text-sm",
          mono && "font-mono text-[13px]",
        )}
        {...props}
      >
        {children}
      </select>
      <ChevronDown
        className="pointer-events-none absolute top-1/2 right-3 size-4 -translate-y-1/2 text-gray-900"
        strokeWidth={1.5}
      />
    </div>
  );
});
