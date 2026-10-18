import type { ClientInfo, SyncReport } from "./api";

/** Human-readable lines for a sync report, one per written file, with reload hints. */
export function describeReport(report: SyncReport, clients: ClientInfo[]): string[] {
  return report.writes.map((w) => {
    const name = clients.find((c) => c.id === w.client)?.name ?? w.client;
    const parts = [
      w.added.length && `added ${w.added.join(", ")}`,
      w.updated.length && `updated ${w.updated.join(", ")}`,
      w.removed.length && `removed ${w.removed.join(", ")}`,
    ].filter(Boolean);
    return `${name}: ${parts.join(" · ")} — ${w.reloadHint}`;
  });
}
