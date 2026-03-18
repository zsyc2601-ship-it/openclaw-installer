import { create } from "zustand";
import { invoke, Channel } from "@tauri-apps/api/core";

export type Phase =
  | "idle"
  | "installing"
  | "awaiting_api_key"
  | "complete"
  | "confirm_uninstall"
  | "uninstalling"
  | "uninstalled"
  | "error";

export interface StepProgress {
  step: number;
  total: number;
  label: string;
  detail: string;
}

interface InstallerState {
  phase: Phase;
  progress: StepProgress;
  error: string | null;
  gatewayUrl: string;

  startInstall: () => Promise<void>;
  submitApiKey: (provider: string, key: string) => Promise<void>;
  startUninstall: (removeData: boolean) => Promise<void>;
  reset: () => void;
}

const initialProgress: StepProgress = {
  step: 0,
  total: 6,
  label: "",
  detail: "",
};

export const useInstaller = create<InstallerState>((set) => ({
  phase: "idle",
  progress: initialProgress,
  error: null,
  gatewayUrl: "http://localhost:18789",

  startInstall: async () => {
    set({ phase: "installing", error: null, progress: initialProgress });

    const onProgress = new Channel<StepProgress>();
    onProgress.onmessage = (msg) => {
      set({ progress: msg });
    };

    try {
      const result = await invoke<string>("start_install", {
        onProgress,
      });

      if (result === "awaiting_api_key") {
        set({ phase: "awaiting_api_key" });
      } else {
        set({ phase: "complete" });
      }
    } catch (e) {
      set({ phase: "error", error: String(e) });
    }
  },

  submitApiKey: async (provider: string, key: string) => {
    try {
      await invoke("save_api_key", { provider, key });
      set({ phase: "complete" });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startUninstall: async (removeData: boolean) => {
    set({ phase: "uninstalling", error: null, progress: initialProgress });

    const onProgress = new Channel<StepProgress>();
    onProgress.onmessage = (msg) => {
      set({ progress: msg });
    };

    try {
      await invoke("start_uninstall", { removeData, onProgress });
      set({ phase: "uninstalled" });
    } catch (e) {
      set({ phase: "error", error: String(e) });
    }
  },

  reset: () => {
    set({ phase: "idle", error: null, progress: initialProgress });
  },
}));
