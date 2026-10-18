import { invoke } from "@tauri-apps/api/core";

export type ClientId =
  | "claude_code"
  | "claude_desktop"
  | "cursor"
  | "vscode"
  | "windsurf"
  | "antigravity"
  | "codex"
  | "gemini_cli";

export type TransportKind = "stdio" | "http";

export interface SettingSpec {
  key: string;
  label: string;
  type: "string" | "boolean";
  required: boolean;
  default?: unknown;
  help?: string;
}

export interface SecretSpec {
  key: string;
  label: string;
  required: boolean;
  help?: string;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  homepage?: string | null;
  tags: string[];
  defaultTransport: TransportKind;
  transports: { stdio?: unknown; http?: unknown };
  settings: SettingSpec[];
  secrets: SecretSpec[];
}

export interface Instance {
  id: string;
  template: string;
  label: string;
  transport: TransportKind;
  settings: Record<string, unknown>;
  secrets: string[];
  createdAt: string;
}

export interface Binding {
  entry: string;
  instance: string;
  clients: ClientId[];
}

export interface Project {
  id: string;
  path: string;
  name: string;
  bindings: Binding[];
}

export interface ClientInfo {
  id: ClientId;
  name: string;
  detected: boolean;
  globalPath: string | null;
  projectFile: string | null;
  reloadHint: string;
}

export interface FileWrite {
  client: ClientId;
  path: string;
  added: string[];
  updated: string[];
  removed: string[];
  reloadHint: string;
  backup: string | null;
}

export interface SyncReport {
  writes: FileWrite[];
}

export interface NewInstance {
  template: string;
  name: string;
  label: string;
  transport?: TransportKind;
  settings: Record<string, unknown>;
  secrets: Record<string, string>;
}

export const api = {
  listTemplates: () => invoke<Template[]>("list_templates"),
  listClients: () => invoke<ClientInfo[]>("list_clients"),
  listInstances: () => invoke<Instance[]>("list_instances"),
  createInstance: (input: NewInstance) => invoke<Instance>("create_instance", { input }),
  deleteInstance: (id: string) => invoke<void>("delete_instance", { id }),
  setSecret: (id: string, key: string, value: string) => invoke<void>("set_secret", { id, key, value }),
  listProjects: () => invoke<Project[]>("list_projects"),
  addProject: (path: string) => invoke<Project>("add_project", { path }),
  removeProject: (id: string) => invoke<SyncReport>("remove_project", { id }),
  setBinding: (projectId: string, entry: string, instance: string, clients: ClientId[]) =>
    invoke<SyncReport>("set_binding", { projectId, entry, instance, clients }),
  removeBinding: (projectId: string, entry: string) =>
    invoke<SyncReport>("remove_binding", { projectId, entry }),
  syncProject: (projectId: string) => invoke<SyncReport>("sync_project", { projectId }),
};

export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
}
