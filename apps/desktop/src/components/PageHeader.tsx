import type { ReactNode } from "react";

export function PageHeader({
  title,
  description,
  actions,
}: {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <div className="flex items-end justify-between gap-6 pb-6">
      <div className="flex min-w-0 flex-col gap-1">
        <h1 className="heading-24">{title}</h1>
        {description && <p className="copy-14 text-gray-900">{description}</p>}
      </div>
      {actions && <div className="flex shrink-0 gap-2">{actions}</div>}
    </div>
  );
}
