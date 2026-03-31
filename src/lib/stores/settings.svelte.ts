import { invoke } from "@tauri-apps/api/core";

export type Settings = {
  dirs: string[];
  exts: string[];
  transcription: boolean;
};

const DEFAULT_SETTINGS: Settings = {
  dirs: [],
  exts: ["mp4", "mkv", "avi", "mov"],
  transcription: false,
};

let _settings = $state<Settings | null>(null);
let _loading = $state(false);
let _promise: Promise<Settings> | null = null;

export const settingsStore = {
  get value() {
    return _settings;
  },
  get loading() {
    return _loading;
  },
  async load(): Promise<Settings> {
    if (_settings !== null) return _settings;
    if (_promise) return _promise;

    _loading = true;
    _promise = invoke<Settings>("get_settings")
      .then((s) => {
        _settings = s;
        return s;
      })
      .catch(() => {
        _settings = { ...DEFAULT_SETTINGS };
        return _settings;
      })
      .finally(() => {
        _loading = false;
        _promise = null;
      });

    return _promise;
  },
  update(patch: Partial<Settings>) {
    if (_settings) {
      _settings = { ..._settings, ...patch };
    }
  },
  async save(s: Settings): Promise<void> {
    await invoke("store_settings", { settings: s });
    _settings = { ...s };
  },
};
