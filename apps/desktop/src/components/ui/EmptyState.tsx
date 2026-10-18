import type { ReactNode } from "react";

export function EmptyState({
  icon,
  title,
  description,
  action,
}: {
  icon: ReactNode;
  title: ReactNode;
  description: ReactNode;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center gap-6 rounded-lg bg-bg-100 px-[70px] py-12 text-center shadow-border">
      <div className="flex size-10 items-center justify-center rounded-lg text-gray-900 shadow-border">{icon}</div>
      <div className="flex max-w-[340px] flex-col gap-1">
        <h3 className="heading-16">{title}</h3>
        <p className="copy-14 text-gray-900">{description}</p>
      </div>
      {action}
    </div>
  );
}
