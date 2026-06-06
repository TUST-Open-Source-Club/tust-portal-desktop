import { invoke } from "@tauri-apps/api/core";

export async function getPlatform(): Promise<string> {
  return await invoke<string>("get_platform");
}
