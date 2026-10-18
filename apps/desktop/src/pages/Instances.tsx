import { useState } from "react";
import { Boxes, KeyRound, Plus, Trash2 } from "lucide-react";
import { NewInstanceModal } from "../components/NewInstanceModal";
import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { EmptyState } from "../components/ui/EmptyState";
import { useToast } from "../components/ui/Toast";
import { api, errorMessage } from "../lib/api";
import { useData } from "../lib/data";

export function Instances() {
  const { instances, templates, projects, reload } = useData();
  const toast = useToast();
  const [creating, setCreating] = useState(false);

  const usage = (id: string) => projects.filter((p) => p.bindings.some((b) => b.instance === id)).map((p) => p.name);

  async function remove(id: string) {
    try {
      await api.deleteInstance(id);
      await reload();
      toast({ tone: "success", title: `Deleted ${id}`, lines: ["Its secrets were removed from the keychain."] });
    } catch (e) {
      toast({ tone: "error", title: "Could not delete instance", lines: [errorMessage(e)] });
    }
  }

  return (
    <>
      <PageHeader
        title="Instances"
        description="Named copies of a server, each with its own settings and credentials."
        actions={
          <Button variant="primary" prefix={<Plus className="size-4" />} onClick={() => setCreating(true)}>
            New instance
          </Button>
        }
      />

      {instances.length === 0 ? (
        <EmptyState
          icon={<Boxes className="size-5" strokeWidth={1.5} />}
          title="No instances yet"
          description="Create one instance per account or project, e.g. supabase@client-a and supabase@personal."
          action={
            <Button variant="primary" onClick={() => setCreating(true)}>
              New instance
            </Button>
          }
        />
      ) : (
        <div className="overflow-hidden rounded-lg bg-bg-100 shadow-border">
          <table className="w-full text-sm">
            <thead>
              <tr className="h-9 text-left text-gray-900 shadow-[inset_0_-1px_0_0_var(--gray-a-400)]">
                <th className="px-4 font-medium">Instance</th>
                <th className="px-2 font-medium">Server</th>
                <th className="px-2 font-medium">Transport</th>
                <th className="px-2 font-medium">Secrets</th>
                <th className="px-2 font-medium">Used in</th>
                <th className="w-12 px-4" />
              </tr>
            </thead>
            <tbody>
              {instances.map((i) => {
                const used = usage(i.id);
                return (
                  <tr
                    key={i.id}
                    className="h-14 shadow-[inset_0_-1px_0_0_var(--gray-a-400)] transition-colors last:shadow-none hover:bg-gray-a-100"
                  >
                    <td className="px-4">
                      <div className="flex flex-col">
                        <span className="font-mono text-[13px]">{i.id}</span>
                        {i.label && <span className="label-12 text-gray-900">{i.label}</span>}
                      </div>
                    </td>
                    <td className="px-2 text-gray-900">{templates.find((t) => t.id === i.template)?.name ?? i.template}</td>
                    <td className="px-2">
                      <Badge tone={i.transport === "http" ? "blue" : "gray"}>{i.transport}</Badge>
                    </td>
                    <td className="px-2">
                      {i.secrets.length ? (
                        <Badge tone="green" icon={<KeyRound className="size-3" />}>
                          {i.secrets.length} in keychain
                        </Badge>
                      ) : (
                        <span className="text-gray-700">—</span>
                      )}
                    </td>
                    <td className="px-2 text-gray-900">{used.length ? used.join(", ") : <span className="text-gray-700">—</span>}</td>
                    <td className="px-4 text-right">
                      <Button
                        variant="tertiary"
                        size="sm"
                        iconOnly
                        aria-label={`Delete ${i.id}`}
                        disabled={used.length > 0}
                        title={used.length ? "Unbind it from projects first" : undefined}
                        onClick={() => remove(i.id)}
                      >
                        <Trash2 className="size-4" strokeWidth={1.5} />
                      </Button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <NewInstanceModal open={creating} onClose={() => setCreating(false)} />
    </>
  );
}
