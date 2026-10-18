import { createContext, useCallback, useContext, useState, type ReactNode } from "react";
import { CircleAlert, CircleCheck, X } from "lucide-react";
import { cn } from "../../lib/cn";

interface ToastItem {
  id: number;
  tone: "success" | "error";
  title: string;
  lines?: string[];
}

const ToastContext = createContext<(t: Omit<ToastItem, "id">) => void>(() => {});

export function useToast() {
  return useContext(ToastContext);
}

let nextId = 1;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);

  const dismiss = useCallback((id: number) => setItems((all) => all.filter((t) => t.id !== id)), []);

  const push = useCallback(
    (t: Omit<ToastItem, "id">) => {
      const id = nextId++;
      setItems((all) => [...all.slice(-3), { ...t, id }]);
      setTimeout(() => dismiss(id), t.tone === "error" ? 8000 : 5000);
    },
    [dismiss],
  );

  return (
    <ToastContext.Provider value={push}>
      {children}
      <div className="pointer-events-none fixed right-4 bottom-4 z-[60] flex w-[356px] flex-col gap-2">
        {items.map((t) => (
          <div key={t.id} className="pointer-events-auto flex gap-3 rounded-xl bg-bg-100 p-4 shadow-menu">
            {t.tone === "success" ? (
              <CircleCheck className="mt-0.5 size-4 shrink-0 text-green-900" strokeWidth={1.5} />
            ) : (
              <CircleAlert className="mt-0.5 size-4 shrink-0 text-red-900" strokeWidth={1.5} />
            )}
            <div className="flex min-w-0 flex-1 flex-col gap-1">
              <p className={cn("label-14 font-medium", t.tone === "error" && "text-red-900")}>{t.title}</p>
              {t.lines?.map((line, i) => (
                <p key={i} className="copy-13 break-words text-gray-900">
                  {line}
                </p>
              ))}
            </div>
            <button className="h-fit text-gray-700 hover:text-gray-1000" aria-label="Dismiss" onClick={() => dismiss(t.id)}>
              <X className="size-4" strokeWidth={1.5} />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}
