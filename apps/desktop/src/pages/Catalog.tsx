import { useState } from "react";
import { Plus, Search } from "lucide-react";
import { NewInstanceModal } from "../components/NewInstanceModal";
import { PageHeader } from "../components/PageHeader";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { useData } from "../lib/data";

export function Catalog() {
  const { templates, instances } = useData();
  const [query, setQuery] = useState("");
  const [creating, setCreating] = useState<string | null>(null);

  const q = query.trim().toLowerCase();
  const shown = templates.filter(
    (t) => !q || t.name.toLowerCase().includes(q) || t.tags.some((tag) => tag.includes(q)) || t.description.toLowerCase().includes(q),
  );

  return (
    <>
      <PageHeader
        title="Catalog"
        description="Server templates. Add your own with a pull request to catalog/servers."
        actions={
          <Input
            className="w-64"
            placeholder="Search servers"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            prefix={<Search className="size-4" strokeWidth={1.5} />}
          />
        }
      />
      <div className="grid grid-cols-3 gap-4">
        {shown.map((t) => {
          const count = instances.filter((i) => i.template === t.id).length;
          return (
            <div key={t.id} className="flex flex-col gap-3 rounded-lg bg-bg-100 p-5 shadow-border">
              <div className="flex items-center justify-between gap-2">
                <h3 className="heading-16">{t.name}</h3>
                {count > 0 && <Badge>{count} instance{count > 1 ? "s" : ""}</Badge>}
              </div>
              <p className="copy-13 flex-1 text-gray-900">{t.description}</p>
              <div className="flex flex-wrap gap-1">
                {t.tags.map((tag) => (
                  <Badge key={tag}>{tag}</Badge>
                ))}
              </div>
              <div className="flex items-center justify-between pt-1">
                {/* TODO: open in the system browser via tauri-plugin-opener */}
                <span className="truncate font-mono label-12 text-gray-700" title={t.homepage ?? undefined}>
                  {t.id}
                </span>
                <Button size="sm" prefix={<Plus className="size-4" />} onClick={() => setCreating(t.id)}>
                  Instance
                </Button>
              </div>
            </div>
          );
        })}
      </div>
      <NewInstanceModal open={creating !== null} templateId={creating ?? undefined} onClose={() => setCreating(null)} />
    </>
  );
}
