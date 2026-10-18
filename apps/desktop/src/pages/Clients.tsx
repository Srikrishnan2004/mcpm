import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/ui/Badge";
import { useData } from "../lib/data";

export function Clients() {
  const { clients } = useData();

  return (
    <>
      <PageHeader title="Clients" description="AI IDEs and CLIs mcpm writes MCP configuration for." />
      <div className="grid grid-cols-2 gap-4">
        {clients.map((c) => (
          <div key={c.id} className="flex flex-col gap-4 rounded-lg bg-bg-100 p-5 shadow-border">
            <div className="flex items-center justify-between">
              <h3 className="heading-16">{c.name}</h3>
              {c.detected ? <Badge tone="green">Detected</Badge> : <Badge>Not found</Badge>}
            </div>
            <dl className="grid grid-cols-[96px_1fr] gap-x-3 gap-y-2 label-13">
              <dt className="text-gray-700">Global</dt>
              <dd className="truncate font-mono text-gray-900" title={c.globalPath ?? undefined}>
                {c.globalPath ?? "—"}
              </dd>
              <dt className="text-gray-700">Project</dt>
              <dd className="font-mono text-gray-900">{c.projectFile ?? "Not supported"}</dd>
              <dt className="text-gray-700">After changes</dt>
              <dd className="text-gray-900">{c.reloadHint}</dd>
            </dl>
          </div>
        ))}
      </div>
    </>
  );
}
