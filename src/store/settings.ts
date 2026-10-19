import { create } from "zustand";
import { api, type Settings } from "../lib/tauri";

interface SettingsState {
  settings: Settings | null;
  error: string | null;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => Promise<boolean>;
}

export const useSettings = create<SettingsState>((set, get) => ({
  settings: null,
  error: null,
  load: async () => set({ settings: await api.getSettings(), error: null }),
  update: async (patch) => {
    const current = get().settings;
    if (!current) return false;
    try {
      const saved = await api.setSettings({ ...current, ...patch });
      set({ settings: saved, error: null });
      return true;
    } catch (e) {
      set({ error: String(e) });
      return false;
    }
  },
}));
