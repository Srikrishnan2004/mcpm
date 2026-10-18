import { ArrowRight, Boxes, FolderGit2, Monitor } from "lucide-react";
import type { ReactNode } from "react";
import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/ui/Badge";
import type { Page } from "../components/Sidebar";
import { useData } from "../lib/data";

export function Overview({ onNavigate }: { onNavigate: (p: Page) => void }) {
  const { instances, projects, clients } = useData();
  const detected = clients.filter((c) => c.detected);

  return (
    <>
      <PageHeader title="Overview" description="Every MCP server, every client, one place." />

      <div className="grid grid-cols-3 gap-4">
        <Stat icon={<Boxes />} label="Instances" value={instances.length} onClick={() => onNavigate("instances")} />
        <Stat icon={<FolderGit2 />} label="Projects" value={projects.length} onClick={() => onNavigate("projects")} />
        <Stat
          icon={<Monitor />}
          label="Clients detected"
          value={`${detected.length} / ${clients.length}`}
          onClick={() => onNavigate("clients")}
        />
      </div>

      <h2 className="heading-16 pt-10 pb-3">Clients on this machine</h2>
      <div className="overflow-hidden rounded-lg bg-bg-100 shadow-border">
        {clients.map((c, i) => (
          <div
            key={c.id}
            className="flex h-12 items-center justify-between gap-4 px-4"
            style={i ? { boxShadow: "inset 0 1px 0 0 var(--gray-a-400)" } : undefined}
          >
            <span className="label-14">{c.name}</span>
            {c.detected ? <Badge tone="green">Detected</Badge> : <Badge>Not found</Badge>}
          </div>
        ))}
      </div>
    </>
  );
}

function Stat({ icon, label, value, onClick }: { icon: ReactNode; label: string; value: ReactNode; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="group flex flex-col gap-3 rounded-lg bg-bg-100 p-5 text-left shadow-border transition-shadow duration-150 hover:shadow-border-hover focus-visible:shadow-focus"
    >
      <div className="flex items-center justify-between text-gray-900 [&_svg]:size-4 [&_svg]:stroke-[1.5]">
        {icon}
        <ArrowRight className="opacity-0 transition-opacity group-hover:opacity-100" />
      </div>
      <div className="flex flex-col">
        <span className="heading-24 tabular-nums">{value}</span>
        <span className="label-13 text-gray-900">{label}</span>
      </div>
    </button>
  );
}
