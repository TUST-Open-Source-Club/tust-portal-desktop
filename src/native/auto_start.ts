import { invoke } from "@tauri-apps/api/core";

export async function getAutoStartEnabled(): Promise<boolean> {
  return await invoke<boolean>("get_auto_start_enabled");
}

export async function setAutoStartEnabled(enabled: boolean): Promise<void> {
  await invoke("set_auto_start_enabled", { enabled });
}
