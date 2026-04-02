import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { IndexStatus } from "./commands";

export function onIndexProgress(
  callback: (status: IndexStatus) => void
): Promise<UnlistenFn> {
  return listen<IndexStatus>("index-progress", (event) => {
    callback(event.payload);
  });
}

export function onIndexComplete(
  callback: (status: IndexStatus) => void
): Promise<UnlistenFn> {
  return listen<IndexStatus>("index-complete", (event) => {
    callback(event.payload);
  });
}

export function onFileChanged(
  callback: (path: string) => void
): Promise<UnlistenFn> {
  return listen<string>("file-changed", (event) => {
    callback(event.payload);
  });
}
