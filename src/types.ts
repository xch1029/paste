export type ClipboardItemType = "text" | "image";

export interface ImageSize {
  width: number;
  height: number;
}

export interface ClipboardItem {
  id: number;
  itemType: ClipboardItemType;
  preview: string;
  isPinned: boolean;
  createdAt: number;
  updatedAt: number;
  imageUrl?: string | null;
  imageSize?: ImageSize | null;
}

export interface HistoryResponse {
  items: ClipboardItem[];
}

export interface AppStatus {
  monitoringPaused: boolean;
  historyCount: number;
  manageHotkey: string;
  pickerHotkey: string;
}

export interface ActionResponse {
  ok: boolean;
  item?: ClipboardItem | null;
  historyCount: number;
}
