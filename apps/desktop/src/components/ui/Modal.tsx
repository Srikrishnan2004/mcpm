import { useEffect, type ReactNode } from "react";
import { X } from "lucide-react";
import { Button } from "./Button";

export function Modal({
  open,
  onClose,
  title,
  description,
  footer,
  width = 520,
  children,
}: {
  open: boolean;
  onClose: () => void;
  title: ReactNode;
  description?: ReactNode;
  footer?: ReactNode;
  width?: number;
  children: ReactNode;
}) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-bg-200/80" onClick={onClose} />
      <div
        role="dialog"
        aria-modal="true"
        className="relative flex max-h-[calc(100vh-32px)] w-full flex-col rounded-xl bg-bg-100 shadow-modal"
        style={{ maxWidth: width }}
      >
        <div className="flex items-start justify-between gap-4 p-6 pb-4">
          <div className="flex flex-col gap-1">
            <h2 className="heading-20">{title}</h2>
            {description && <p className="copy-14 text-gray-900">{description}</p>}
          </div>
          <Button variant="tertiary" size="sm" iconOnly aria-label="Close" onClick={onClose}>
            <X className="size-4" strokeWidth={1.5} />
          </Button>
        </div>
        <div className="overflow-y-auto px-6 pb-6">{children}</div>
        {footer && (
          <div className="flex justify-end gap-2 rounded-b-xl bg-bg-200 px-6 py-4 shadow-[inset_0_1px_0_0_var(--gray-a-400)]">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}
