import { create } from "zustand";
import { api, isActive, type Job } from "../lib/tauri";

interface JobsState {
  jobs: Job[];
  /** substitui a lista inteira (carga inicial) */
  setAll: (jobs: Job[]) => void;
  /** aplica uma atualização vinda do evento `jobs://update` */
  upsert: (job: Job) => void;
  clearFinished: () => Promise<void>;
  remove: (id: string) => Promise<void>;
  /** progresso do download inicial do yt-dlp (0–1), ou null */
  ytdlpInstall: number | null;
  setYtdlpInstall: (p: number | null) => void;
}

export const useJobs = create<JobsState>((set) => ({
  jobs: [],
  setAll: (jobs) => set({ jobs }),
  upsert: (job) =>
    set((s) => {
      const i = s.jobs.findIndex((j) => j.id === job.id);
      if (i === -1) return { jobs: [job, ...s.jobs] };
      const jobs = s.jobs.slice();
      jobs[i] = job;
      return { jobs };
    }),
  clearFinished: async () => set({ jobs: await api.clearFinished() }),
  remove: async (id) => {
    if (await api.removeJob(id)) set((s) => ({ jobs: s.jobs.filter((j) => j.id !== id) }));
  },
  ytdlpInstall: null,
  setYtdlpInstall: (ytdlpInstall) => set({ ytdlpInstall }),
}));

export const selectHasFinished = (s: JobsState) => s.jobs.some((j) => !isActive(j.stage));
export const selectActiveCount = (s: JobsState) => s.jobs.filter((j) => isActive(j.stage)).length;
