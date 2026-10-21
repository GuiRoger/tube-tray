import { create } from "zustand";
import { api, type Profile, type Settings } from "../lib/tauri";

interface SettingsState {
  settings: Settings | null;
  /** formatos disponíveis (vêm do núcleo) */
  profiles: Profile[];
  error: string | null;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => Promise<boolean>;
}

export const useSettings = create<SettingsState>((set, get) => ({
  settings: null,
  profiles: [],
  error: null,
  load: async () => {
    const [settings, profiles] = await Promise.all([api.getSettings(), api.profiles().catch(() => [])]);
    set({ settings, profiles, error: null });
  },
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

/** Perfil escolhido (ou o primeiro, se o id salvo não existir mais). */
export const selectProfile = (s: SettingsState): Profile | null =>
  s.profiles.find((p) => p.id === s.settings?.profileId) ?? s.profiles[0] ?? null;
