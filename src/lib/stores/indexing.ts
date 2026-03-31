import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

export type IndexProgress = {
  current: number;
  total: number;
  file: string;
};

export const indexing = writable(false);
export const progress = writable<IndexProgress | null>(null);

let listenersInitialized = false;

export const initIndexingEvents = async () => {
  if (listenersInitialized) return;
  listenersInitialized = true;

  await listen("init:start", () => {
    indexing.set(true);
    progress.set(null);
  });

  await listen<IndexProgress>("init:progress", (event) => {
    indexing.set(true);
    progress.set(event.payload);
  });

  await listen("init:done", () => {
    indexing.set(false);
  });
};

export const runIndex = async () => {
  await initIndexingEvents();
  indexing.set(true);
  progress.set(null);

  try {
    await invoke("index");
  } catch (error) {
    indexing.set(false);
    throw error;
  }
};
