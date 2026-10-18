import { useEffect, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { FolderGit2, FolderPlus, Plus, RefreshCw, Trash2, Unlink } from "lucide-react";
import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { EmptyState } from "../components/ui/EmptyState";
import { Field, Input } from "../components/ui/Input";
import { Modal } from "../components/ui/Modal";
import { Select } from "../components/ui/Select";
import { Checkbox } from "../components/ui/Toggle";
import { useToast } from "../components/ui/Toast";
import { api, errorMessage, type Binding, type ClientId, type Instance, type Project, type SyncReport } from "../lib/api";
import { cn } from "../lib/cn";
import { useData } from "../lib/data";
import { describeReport } from "../lib/report";

export function Projects() {
  const { projects, instances, clients, reload } = useData();
  const toast = useToast();
  const [selected, setSelected] = useState<string | null>(projects[0]?.id ?? null);
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    if (!projects.some((p) => p.id === selected)) setSelected(projects[0]?.id ?? null);
  }, [projects, selected]);

  const project = projects.find((p) => p.id === selected);

  async function run(action: () => Promise<SyncReport>, title: string) {
    try {
      const report = await action();
      await reload();
      const lines = describeReport(report, clients);
      toast({ tone: "success", title, lines: lines.length ? lines : ["Already in sync, nothing written."] });
    } catch (e) {
      toast({ tone: "error", title: "Sync failed", lines: [errorMessage(e)] });
    }
  }

  async function addProject() {
    const path = await openDialog({ directory: true, multiple: false, title: "Choose a project folder" });
    if (typeof path !== "string") return;
    try {
      const p = await api.addProject(path);
      await reload();
      setSelected(p.id);
    } catch (e) {
      toast({ tone: "error", title: "Could not add project", lines: [errorMessage(e)] });
    }
  }

  return (
    <>
      <PageHeader
        title="Projects"
        description="Bind servers per folder. Remap an entry to another instance any time."
        actions={
          <Button variant="primary" prefix={<FolderPlus className="size-4" />} onClick={addProject}>
            Add project
          </Button>
        }
      />

      {projects.length === 0 ? (
        <EmptyState
          icon={<FolderGit2 className="size-5" strokeWidth={1.5} />}
          title="No projects yet"
          description="Add a project folder, then choose which instance each server entry points to."
          action={
            <Button variant="primary" onClick={addProject}>
              Add project
            </Button>
          }
        />
      ) : (
        <div className="grid grid-cols-[220px_1fr] gap-6">
          <nav className="flex flex-col gap-0.5">
            {projects.map((p) => (
              <button
                key={p.id}
                onClick={() => setSelected(p.id)}
                className={cn(
                  "flex h-9 items-center justify-between gap-2 rounded-md px-2.5 text-left text-sm transition-colors duration-150 focus-visible:shadow-focus",
                  p.id === selected ? "bg-gray-a-200 font-medium" : "text-gray-900 hover:bg-gray-a-100 hover:text-gray-1000",
                )}
              >
                <span className="truncate">{p.name}</span>
                <span className="label-12 text-gray-700 tabular-nums">{p.bindings.length}</span>
              </button>
            ))}
          </nav>

          {project && (
            <section className="flex min-w-0 flex-col gap-4">
              <div className="flex items-start justify-between gap-4">
                <div className="flex min-w-0 flex-col gap-0.5">
                  <h2 className="heading-20">{project.name}</h2>
                  <span className="truncate font-mono label-13 text-gray-900">{project.path}</span>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    prefix={<RefreshCw className="size-3.5" />}
                    onClick={() => run(() => api.syncProject(project.id), "Project synced")}
                  >
                    Sync
                  </Button>
                  <Button
                    size="sm"
                    iconOnly
                    aria-label="Remove project"
                    title="Remove project (removes mcpm's entries from its config files)"
                    onClick={() => run(() => api.removeProject(project.id), `Removed ${project.name}`)}
                  >
                    <Trash2 className="size-4" strokeWidth={1.5} />
                  </Button>
                  <Button size="sm" variant="primary" prefix={<Plus className="size-4" />} onClick={() => setAdding(true)}>
                    Add server
                  </Button>
                </div>
              </div>

              <BindingsTable project={project} instances={instances} run={run} />
              <AddBindingModal open={adding} onClose={() => setAdding(false)} project={project} run={run} />
            </section>
          )}
        </div>
      )}
    </>
  );
}

function BindingsTable({
  project,
  instances,
  run,
}: {
  project: Project;
  instances: Instance[];
  run: (action: () => Promise<SyncReport>, title: string) => Promise<void>;
}) {
  const { clients } = useData();

  if (project.bindings.length === 0) {
    return (
      <div className="rounded-lg bg-bg-100 px-6 py-10 text-center copy-14 text-gray-900 shadow-border">
        No servers bound yet. Use <span className="font-medium text-gray-1000">Add server</span> to write one into this
        project's client configs.
      </div>
    );
  }

  const remap = (b: Binding, instance: string) =>
    run(() => api.setBinding(project.id, b.entry, instance, b.clients), `Remapped ${b.entry} → ${instance}`);

  return (
    <div className="overflow-hidden rounded-lg bg-bg-100 shadow-border">
      <table className="w-full text-sm">
        <thead>
          <tr className="h-9 text-left text-gray-900 shadow-[inset_0_-1px_0_0_var(--gray-a-400)]">
            <th className="w-36 px-4 font-medium">Entry</th>
            <th className="px-2 font-medium">Instance</th>
            <th className="px-2 font-medium">Clients</th>
            <th className="w-12 px-4" />
          </tr>
        </thead>
        <tbody>
          {project.bindings.map((b) => {
            const current = instances.find((i) => i.id === b.instance);
            const options = instances.filter((i) => !current || i.template === current.template);
            return (
              <tr key={b.entry} className="h-14 shadow-[inset_0_-1px_0_0_var(--gray-a-400)] last:shadow-none">
                <td className="px-4 font-mono text-[13px]">{b.entry}</td>
                <td className="px-2">
                  <Select size_="sm" mono className="max-w-72" value={b.instance} onChange={(e) => remap(b, e.target.value)}>
                    {!current && <option value={b.instance}>{b.instance} (missing)</option>}
                    {options.map((i) => (
                      <option key={i.id} value={i.id}>
                        {i.id}
                        {i.label ? ` — ${i.label}` : ""}
                      </option>
                    ))}
                  </Select>
                </td>
                <td className="px-2">
                  <div className="flex flex-wrap gap-1">
                    {b.clients.map((c) => (
                      <Badge key={c}>{clients.find((x) => x.id === c)?.name ?? c}</Badge>
                    ))}
                  </div>
                </td>
                <td className="px-4 text-right">
                  <Button
                    variant="tertiary"
                    size="sm"
                    iconOnly
                    aria-label={`Unbind ${b.entry}`}
                    onClick={() => run(() => api.removeBinding(project.id, b.entry), `Unbound ${b.entry}`)}
                  >
                    <Unlink className="size-4" strokeWidth={1.5} />
                  </Button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function AddBindingModal({
  open,
  onClose,
  project,
  run,
}: {
  open: boolean;
  onClose: () => void;
  project: Project;
  run: (action: () => Promise<SyncReport>, title: string) => Promise<void>;
}) {
  const { instances, clients } = useData();
  const projectClients = clients.filter((c) => c.projectFile);
  const [instance, setInstance] = useState("");
  const [entry, setEntry] = useState("");
  const [targets, setTargets] = useState<ClientId[]>([]);

  useEffect(() => {
    if (!open) return;
    const first = instances[0];
    setInstance(first?.id ?? "");
    setEntry(first?.template ?? "");
    setTargets(projectClients.filter((c) => c.detected).map((c) => c.id));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const valid = /^[a-z0-9][a-z0-9-]*$/.test(entry) && instance && targets.length > 0;
  const taken = project.bindings.some((b) => b.entry === entry);

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Add server to project"
      description={`Writes an mcpm entry into ${project.name}'s client config files. No secrets are written.`}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button
            variant="primary"
            disabled={!valid}
            onClick={async () => {
              await run(() => api.setBinding(project.id, entry, instance, targets), `Bound ${entry} → ${instance}`);
              onClose();
            }}
          >
            {taken ? "Remap" : "Add"}
          </Button>
        </>
      }
    >
      {instances.length === 0 ? (
        <p className="copy-14 text-gray-900">Create an instance first on the Instances page.</p>
      ) : (
        <div className="flex flex-col gap-4">
          <Field label="Instance">
            <Select
              mono
              value={instance}
              onChange={(e) => {
                setInstance(e.target.value);
                const t = instances.find((i) => i.id === e.target.value)?.template;
                if (t) setEntry(t);
              }}
            >
              {instances.map((i) => (
                <option key={i.id} value={i.id}>
                  {i.id}
                </option>
              ))}
            </Select>
          </Field>
          <Field
            label="Entry name"
            help={taken ? "This entry already exists in the project and will be remapped." : "The server name the IDE sees. Keep it stable across projects."}
          >
            <Input mono value={entry} onChange={(e) => setEntry(e.target.value.toLowerCase())} />
          </Field>
          <div className="flex flex-col gap-2">
            <span className="label-13 text-gray-900">Write to</span>
            <div className="grid grid-cols-2 gap-2">
              {projectClients.map((c) => (
                <Checkbox
                  key={c.id}
                  checked={targets.includes(c.id)}
                  onChange={(on) => setTargets((t) => (on ? [...t, c.id] : t.filter((x) => x !== c.id)))}
                >
                  <span className="flex flex-col">
                    <span>{c.name}</span>
                    <span className="font-mono label-12 text-gray-700">{c.projectFile}</span>
                  </span>
                </Checkbox>
              ))}
            </div>
          </div>
        </div>
      )}
    </Modal>
  );
}
