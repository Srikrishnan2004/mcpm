import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";
import { api, errorMessage, type ClientInfo, type Instance, type Project, type Template } from "./api";

interface Data {
  templates: Template[];
  instances: Instance[];
  projects: Project[];
  clients: ClientInfo[];
  loading: boolean;
  error: string | null;
  reload: () => Promise<void>;
}

const DataContext = createContext<Data | null>(null);

export function DataProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<Omit<Data, "reload">>({
    templates: [],
    instances: [],
    projects: [],
    clients: [],
    loading: true,
    error: null,
  });

  const reload = useCallback(async () => {
    try {
      const [templates, instances, projects, clients] = await Promise.all([
        api.listTemplates(),
        api.listInstances(),
        api.listProjects(),
        api.listClients(),
      ]);
      setState({ templates, instances, projects, clients, loading: false, error: null });
    } catch (e) {
      setState((s) => ({ ...s, loading: false, error: errorMessage(e) }));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return <DataContext.Provider value={{ ...state, reload }}>{children}</DataContext.Provider>;
}

export function useData(): Data {
  const ctx = useContext(DataContext);
  if (!ctx) throw new Error("useData must be used inside <DataProvider>");
  return ctx;
}
