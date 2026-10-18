import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";
import { Loader2 } from "lucide-react";
import { cn } from "../../lib/cn";

type Variant = "primary" | "secondary" | "tertiary" | "error" | "warning";
type Size = "sm" | "md" | "lg";

const variants: Record<Variant, string> = {
  primary: "bg-gray-1000 text-bg-100 hover:bg-[var(--primary-hover)]",
  secondary: "bg-bg-100 text-gray-1000 shadow-border hover:bg-gray-a-200",
  tertiary: "bg-transparent text-gray-1000 hover:bg-gray-a-200",
  error: "bg-red-800 text-white hover:bg-red-900",
  warning: "bg-amber-800 text-[#0a0a0a] hover:bg-[#d27504]",
};

const sizes: Record<Size, string> = {
  sm: "h-8 px-2.5 gap-1.5 rounded-md text-sm",
  md: "h-9 px-3.5 gap-2 rounded-md text-sm",
  lg: "h-10 px-4 gap-2 rounded-lg text-base",
};

const iconOnlySizes: Record<Size, string> = {
  sm: "size-8 rounded-md",
  md: "size-9 rounded-md",
  lg: "size-10 rounded-lg",
};

export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "prefix"> {
  variant?: Variant;
  size?: Size;
  prefix?: ReactNode;
  suffix?: ReactNode;
  loading?: boolean;
  /** Square icon button; pass `aria-label`. */
  iconOnly?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "secondary", size = "md", prefix, suffix, loading, iconOnly, className, children, disabled, ...props },
  ref,
) {
  return (
    <button
      ref={ref}
      type="button"
      disabled={disabled || loading}
      className={cn(
        "inline-flex shrink-0 select-none items-center justify-center font-medium whitespace-nowrap",
        "transition-[background-color,box-shadow,color] duration-150",
        "focus-visible:shadow-focus",
        "disabled:cursor-not-allowed disabled:bg-gray-100 disabled:text-gray-700 disabled:shadow-border",
        variants[variant],
        iconOnly ? iconOnlySizes[size] : sizes[size],
        className,
      )}
      {...props}
    >
      {loading ? <Loader2 className="size-4 animate-spin" /> : prefix}
      {children}
      {suffix}
    </button>
  );
});
