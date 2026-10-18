import { useEffect, useMemo, useState } from "react";
import { api, errorMessage, type Template } from "../lib/api";
import { useData } from "../lib/data";
import { Button } from "./ui/Button";
import { Field, Input, SecretInput } from "./ui/Input";
import { Modal } from "./ui/Modal";
import { Select } from "./ui/Select";
import { Toggle } from "./ui/Toggle";
import { useToast } from "./ui/Toast";

const SLUG = /^[a-z0-9][a-z0-9-]*$/;

function initialSettings(t: Template | undefined): Record<string, unknown> {
  return Object.fromEntries((t?.settings ?? []).map((s) => [s.key, s.default ?? (s.type === "boolean" ? false : "")]));
}

export function NewInstanceModal({
  open,
  onClose,
  templateId,
}: {
  open: boolean;
  onClose: () => void;
  templateId?: string;
}) {
  const { templates, reload } = useData();
  const toast = useToast();
  const [template, setTemplate] = useState(templateId ?? templates[0]?.id ?? "");
  const [name, setName] = useState("");
  const [label, setLabel] = useState("");
  const [settings, setSettings] = useState<Record<string, unknown>>({});
  const [secrets, setSecrets] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const t = useMemo(() => templates.find((x) => x.id === template), [templates, template]);

  useEffect(() => {
    if (open) {
      const id = templateId ?? templates[0]?.id ?? "";
      setTemplate(id);
      setName("");
      setLabel("");
      setSecrets({});
      setError(null);
      setSettings(initialSettings(templates.find((x) => x.id === id)));
    }
  }, [open, templateId, templates]);

  const nameValid = SLUG.test(name);
  const missing =
    !t ||
    t.settings.some((s) => s.required && s.type === "string" && !String(settings[s.key] ?? "").trim()) ||
    t.secrets.some((s) => s.required && !secrets[s.key]?.trim());

  async function submit() {
    if (!t) return;
    setSaving(true);
    setError(null);
    try {
      const inst = await api.createInstance({ template: t.id, name, label, settings, secrets });
      await reload();
      toast({ tone: "success", title: `Created ${inst.id}`, lines: ["Secrets were saved to the OS keychain."] });
      onClose();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="New instance"
      description="A named copy of a server with its own settings and credentials."
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button variant="primary" loading={saving} disabled={!nameValid || missing} onClick={submit}>
            Create instance
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <Field label="Server">
          <Select
            value={template}
            onChange={(e) => {
              setTemplate(e.target.value);
              setSettings(initialSettings(templates.find((x) => x.id === e.target.value)));
              setSecrets({});
            }}
            disabled={!!templateId}
          >
            {templates.map((x) => (
              <option key={x.id} value={x.id}>
                {x.name}
              </option>
            ))}
          </Select>
        </Field>

        <Field
          label="Instance name"
          help={name ? `ID: ${template}@${name}` : "Lowercase letters, digits and dashes, e.g. client-a"}
          error={name && !nameValid ? "Use lowercase letters, digits and dashes" : undefined}
        >
          <Input
            mono
            autoFocus
            value={name}
            placeholder="client-a"
            prefix={<span className="font-mono text-[13px]">{template}@</span>}
            onChange={(e) => setName(e.target.value.toLowerCase())}
            invalid={!!name && !nameValid}
          />
        </Field>

        <Field label="Label (optional)">
          <Input value={label} placeholder="Client A – production" onChange={(e) => setLabel(e.target.value)} />
        </Field>

        {t?.settings.map((s) =>
          s.type === "boolean" ? (
            <div key={s.key} className="flex items-center justify-between gap-4">
              <div className="flex flex-col">
                <span className="label-14">{s.label}</span>
                {s.help && <span className="label-13 text-gray-700">{s.help}</span>}
              </div>
              <Toggle
                label={s.label}
                checked={Boolean(settings[s.key])}
                onChange={(v) => setSettings((x) => ({ ...x, [s.key]: v }))}
              />
            </div>
          ) : (
            <Field key={s.key} label={s.label + (s.required ? "" : " (optional)")} help={s.help}>
              <Input
                mono
                value={String(settings[s.key] ?? "")}
                onChange={(e) => setSettings((x) => ({ ...x, [s.key]: e.target.value }))}
              />
            </Field>
          ),
        )}

        {t?.secrets.map((s) => (
          <Field key={s.key} label={s.label} help={s.help ?? "Stored in the OS keychain, never in config files."}>
            <SecretInput
              value={secrets[s.key] ?? ""}
              onChange={(e) => setSecrets((x) => ({ ...x, [s.key]: e.target.value }))}
            />
          </Field>
        ))}

        {error && <p className="copy-13 rounded-md bg-red-100 px-3 py-2 text-red-900">{error}</p>}
      </div>
    </Modal>
  );
}
