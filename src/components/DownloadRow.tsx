import { memo } from "react";
import { api, type Job } from "../lib/tauri";
import { formatPercent, formatRemaining, formatSize, formatSpeed } from "../lib/format";
import { revealLabel } from "../lib/platform";
import { useJobs } from "../store/jobs";
import { Icon, type IconName } from "./Icon";

function statusText(job: Job, ytdlpInstall: number | null): string {
  switch (job.stage) {
    case "queued":
      return "Na fila";
    case "starting":
      return ytdlpInstall != null
        ? `Instalando o yt-dlp… ${Math.floor(ytdlpInstall * 100)}%`
        : "Preparando…";
    case "downloading":
      return [formatPercent(job.percent), formatSpeed(job.speed), formatRemaining(job.eta)]
        .filter(Boolean)
        .join(" · ");
    case "merging":
      return "Finalizando…";
    case "done":
      if (job.missing) return "Arquivo excluído";
      return ["Concluído", formatSize(job.size)].filter(Boolean).join(" · ");
    case "failed":
      return `Falhou · ${job.error ?? "erro desconhecido"}`;
    case "canceled":
      return "Cancelado";
  }
}

const ROW_ICON: Record<Job["stage"], { name: IconName; cls: string }> = {
  queued: { name: "download", cls: "downloading" },
  starting: { name: "download", cls: "downloading" },
  downloading: { name: "download", cls: "downloading" },
  merging: { name: "download", cls: "downloading" },
  done: { name: "check-circle", cls: "done" },
  failed: { name: "exclamation", cls: "failed" },
  canceled: { name: "xmark", cls: "canceled" },
};

function Action({ icon, label, onClick }: { icon: IconName; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      className="icon-button"
      aria-label={label}
      title={label}
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
    >
      <Icon name={icon} />
    </button>
  );
}

/** §7.7 — uma linha da lista de downloads. */
export const DownloadRow = memo(function DownloadRow({ job }: { job: Job }) {
  const ytdlpInstall = useJobs((s) => s.ytdlpInstall);
  const remove = useJobs((s) => s.remove);
  const { stage } = job;
  const active = stage === "queued" || stage === "starting" || stage === "downloading" || stage === "merging";
  const deleted = stage === "done" && job.missing;
  const icon = deleted ? { name: "trash" as IconName, cls: "deleted" } : ROW_ICON[stage];
  const status = statusText(job, ytdlpInstall);
  const title = job.title ?? job.url.replace(/^https?:\/\/(www\.)?/, "");

  const open = () => job.file && api.openFile(job.file);

  return (
    <div
      className={deleted ? "row is-deleted" : "row"}
      onDoubleClick={stage === "done" && !deleted ? open : undefined}
      aria-label={`${title}, ${status}`}
    >
      <span className={`row-icon ${icon.cls}`}>
        <Icon name={icon.name} />
      </span>

      <div className="row-body">
        <div className="row-title" title={title}>
          {title}
        </div>
        {active && (
          <div
            className={`progress${stage === "starting" || stage === "merging" ? " indeterminate" : ""}`}
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.floor(job.percent)}
          >
            <div style={{ width: `${job.percent}%` }} />
          </div>
        )}
        <div className={`row-status${stage === "failed" ? " error" : ""}`} title={status}>
          {status}
        </div>
      </div>

      <div className="row-actions">
        {active && <Action icon="xmark" label="Cancelar download" onClick={() => api.cancel(job.id)} />}
        {stage === "done" && job.file && !deleted && (
          <>
            <Action icon="magnifier" label={revealLabel} onClick={() => api.revealFile(job.file!)} />
            <Action icon="play" label="Abrir" onClick={open} />
          </>
        )}
        {(stage === "failed" || stage === "canceled") && (
          <Action icon="retry" label="Tentar de novo" onClick={() => api.retry(job.id)} />
        )}
        {deleted && <Action icon="retry" label="Baixar de novo" onClick={() => api.retry(job.id)} />}
        {!active && <Action icon="trash" label="Remover da lista" onClick={() => remove(job.id)} />}
      </div>
    </div>
  );
});
