import { forwardRef, useState, type InputHTMLAttributes, type ReactNode } from "react";
import { Eye, EyeOff, KeyRound } from "lucide-react";
import { cn } from "../../lib/cn";

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "prefix"> {
  prefix?: ReactNode;
  suffix?: ReactNode;
  mono?: boolean;
  invalid?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { prefix, suffix, mono, invalid, className, disabled, ...props },
  ref,
) {
  return (
    <div
      className={cn(
        "flex h-9 w-full items-stretch overflow-hidden rounded-md bg-bg-100 shadow-border",
        "transition-shadow duration-150 hover:shadow-border-hover focus-within:shadow-input-focus! ",
        invalid && "shadow-[0_0_0_1px_var(--red-900),0_0_0_4px_color-mix(in_srgb,var(--red-900)_16%,transparent)]!",
        disabled && "bg-gray-100",
        className,
      )}
    >
      {prefix && (
        <span className="flex items-center px-3 text-gray-700 shadow-[1px_0_0_0_var(--gray-a-400)]">{prefix}</span>
      )}
      <input
        ref={ref}
        disabled={disabled}
        className={cn(
          "min-w-0 flex-1 bg-transparent px-3 text-sm text-gray-1000 outline-none placeholder:text-gray-700",
          "disabled:cursor-not-allowed disabled:text-gray-700",
          mono && "font-mono text-[13px]",
        )}
        {...props}
      />
      {suffix && (
        <span className="flex items-center px-3 text-gray-700 shadow-[-1px_0_0_0_var(--gray-a-400)]">{suffix}</span>
      )}
    </div>
  );
});

/** Masked input for credentials. Values go straight to the OS keychain. */
export function SecretInput(props: Omit<InputProps, "type" | "prefix" | "suffix">) {
  const [revealed, setRevealed] = useState(false);
  return (
    <Input
      {...props}
      mono
      type={revealed ? "text" : "password"}
      autoComplete="off"
      spellCheck={false}
      prefix={<KeyRound className="size-4" strokeWidth={1.5} />}
      suffix={
        <button
          type="button"
          className="text-gray-700 hover:text-gray-1000"
          aria-label={revealed ? "Hide value" : "Show value"}
          onClick={() => setRevealed((r) => !r)}
        >
          {revealed ? <EyeOff className="size-4" strokeWidth={1.5} /> : <Eye className="size-4" strokeWidth={1.5} />}
        </button>
      }
    />
  );
}

export function Field({
  label,
  help,
  error,
  children,
}: {
  label: ReactNode;
  help?: ReactNode;
  error?: ReactNode;
  children: ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="label-13 text-gray-900">{label}</span>
      {children}
      {error ? (
        <span className="label-13 text-red-900">{error}</span>
      ) : help ? (
        <span className="label-13 text-gray-700">{help}</span>
      ) : null}
    </label>
  );
}
