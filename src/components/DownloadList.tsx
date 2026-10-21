import { useState } from "react";
import { api } from "../lib/tauri";
import { selectActiveCount, useJobs } from "../store/jobs";
import { DownloadRow } from "./DownloadRow";
import { Icon } from "./Icon";

const RECENT_LIMIT = 5;

/** Lista "Recentes" (máx. 5 + "Ver todos") e estado vazio §7.8. */
export function DownloadList() {
  const jobs = useJobs((s) => s.jobs);
  const [showAll, setShowAll] = useState(false);
  const activeCount = useJobs(selectActiveCount);

  if (jobs.length === 0) {
    return (
      <div className="empty">
        <Icon name="tray-download" />
        <p>Nenhum download ainda</p>
        <small>Copie um link do YouTube e ele aparece aqui.</small>
      </div>
    );
  }

  const visible = showAll ? jobs : jobs.slice(0, RECENT_LIMIT);
  const hidden = jobs.length - RECENT_LIMIT;

  return (
    <div className="list">
      <div className="list-header">
        <span>{activeCount > 1 ? `Recentes · ${activeCount} ativos` : "Recentes"}</span>
        <span className="list-actions">
          {activeCount > 1 && (
            <button type="button" className="link-button" onClick={() => api.cancelAll()}>
              Cancelar todos
            </button>
          )}
          {hidden > 0 && (
            <button type="button" className="link-button" onClick={() => setShowAll((v) => !v)}>
              {showAll ? "Ver menos" : `Ver todos (${jobs.length})`}
            </button>
          )}
        </span>
      </div>
      {visible.map((job) => (
        <DownloadRow key={job.id} job={job} />
      ))}
    </div>
  );
}
