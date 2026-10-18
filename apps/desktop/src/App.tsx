import { useState } from "react";
import { Loader2 } from "lucide-react";
import { Sidebar, type Page } from "./components/Sidebar";
import { useData } from "./lib/data";
import { Catalog } from "./pages/Catalog";
import { Clients } from "./pages/Clients";
import { Instances } from "./pages/Instances";
import { Overview } from "./pages/Overview";
import { Projects } from "./pages/Projects";

export function App() {
  const [page, setPage] = useState<Page>("overview");
  const { loading, error } = useData();

  return (
    <div className="flex h-full">
      <Sidebar page={page} onNavigate={setPage} />
      <main className="min-w-0 flex-1 overflow-y-auto bg-bg-100">
        <div className="mx-auto max-w-[1200px] px-8 py-8">
          {loading ? (
            <div className="flex justify-center py-24 text-gray-700">
              <Loader2 className="size-5 animate-spin" />
            </div>
          ) : error ? (
            <p className="copy-14 rounded-md bg-red-100 px-4 py-3 text-red-900">{error}</p>
          ) : (
            {
              overview: <Overview onNavigate={setPage} />,
              projects: <Projects />,
              instances: <Instances />,
              clients: <Clients />,
              catalog: <Catalog />,
            }[page]
          )}
        </div>
      </main>
    </div>
  );
}
