import { useState } from "react";
import { Boxes, FolderGit2, LayoutGrid, Library, Monitor, Moon, Sun, type LucideIcon } from "lucide-react";
import { cn } from "../lib/cn";
import { getThemePref, setThemePref, type ThemePref } from "../lib/theme";

export type Page = "overview" | "instances" | "projects" | "clients" | "catalog";

const items: { id: Page; label: string; icon: LucideIcon }[] = [
  { id: "overview", label: "Overview", icon: LayoutGrid },
  { id: "projects", label: "Projects", icon: FolderGit2 },
  { id: "instances", label: "Instances", icon: Boxes },
  { id: "clients", label: "Clients", icon: Monitor },
  { id: "catalog", label: "Catalog", icon: Library },
];

const themes: { id: ThemePref; icon: LucideIcon; label: string }[] = [
  { id: "system", icon: Monitor, label: "System theme" },
  { id: "light", icon: Sun, label: "Light theme" },
  { id: "dark", icon: Moon, label: "Dark theme" },
];

export function Sidebar({ page, onNavigate }: { page: Page; onNavigate: (p: Page) => void }) {
  const [theme, setTheme] = useState(getThemePref);

  return (
    <aside className="flex w-60 shrink-0 flex-col bg-bg-200 shadow-[inset_-1px_0_0_0_var(--gray-a-400)]">
      <div className="flex h-14 items-center gap-2 px-4">
        <Logo />
        <span className="font-mono text-[15px] font-semibold tracking-tight">mcpm</span>
      </div>

      <nav className="flex flex-1 flex-col gap-0.5 px-2 pt-2">
        {items.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => onNavigate(id)}
            aria-current={page === id ? "page" : undefined}
            className={cn(
              "flex h-8 items-center gap-2.5 rounded-md px-2.5 text-sm transition-colors duration-150 focus-visible:shadow-focus",
              page === id
                ? "bg-gray-a-200 font-medium text-gray-1000"
                : "text-gray-900 hover:bg-gray-a-100 hover:text-gray-1000",
            )}
          >
            <Icon className="size-4" strokeWidth={1.5} />
            {label}
          </button>
        ))}
      </nav>

      <div className="flex items-center justify-between px-4 py-3 shadow-[inset_0_1px_0_0_var(--gray-a-400)]">
        <span className="label-12 text-gray-700">v0.1.0</span>
        <div className="flex rounded-full p-0.5 shadow-border" role="radiogroup" aria-label="Theme">
          {themes.map(({ id, icon: Icon, label }) => (
            <button
              key={id}
              role="radio"
              aria-checked={theme === id}
              aria-label={label}
              onClick={() => {
                setTheme(id);
                setThemePref(id);
              }}
              className={cn(
                "flex size-6 items-center justify-center rounded-full transition-colors duration-150",
                theme === id ? "bg-gray-a-200 text-gray-1000" : "text-gray-700 hover:text-gray-1000",
              )}
            >
              <Icon className="size-3.5" strokeWidth={1.5} />
            </button>
          ))}
        </div>
      </div>
    </aside>
  );
}

function Logo() {
  return (
    <svg viewBox="0 0 1024 1024" className="size-6" aria-hidden>
      <rect width="1024" height="1024" rx="224" className="fill-gray-1000" />
      <g className="stroke-bg-100" strokeWidth="44" strokeLinecap="round" fill="none">
        <line x1="512" y1="512" x2="512" y2="262" />
        <line x1="512" y1="512" x2="296" y2="637" />
        <line x1="512" y1="512" x2="728" y2="637" />
      </g>
      <circle cx="512" cy="512" r="96" className="fill-bg-100" />
      {[
        [512, 232],
        [270, 652],
        [754, 652],
      ].map(([cx, cy]) => (
        <circle key={cx} cx={cx} cy={cy} r="64" className="fill-gray-1000 stroke-bg-100" strokeWidth="44" />
      ))}
    </svg>
  );
}
