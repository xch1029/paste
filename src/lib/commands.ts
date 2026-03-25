import { invoke } from "@tauri-apps/api/core";
import type {
  ActionResponse,
  AppStatus,
  HistoryResponse,
} from "../types";

export function getHistory(query: string) {
  return invoke<HistoryResponse>("get_history", {
    query: query.trim() ? query : null,
  });
}

export function getAppState() {
  return invoke<AppStatus>("get_app_state");
}

export function copyItem(id: number) {
  return invoke<ActionResponse>("copy_item_to_clipboard", { id });
}

export function toggleItemPin(id: number) {
  return invoke<ActionResponse>("toggle_item_pin", { id });
}

export function deleteItem(id: number) {
  return invoke<ActionResponse>("delete_item", { id });
}

export function clearHistory() {
  return invoke<ActionResponse>("clear_history");
}

export function setMonitoringPaused(paused: boolean) {
  return invoke<AppStatus>("set_monitoring_paused", { paused });
}

export function hidePanel() {
  return invoke("hide_panel");
}
