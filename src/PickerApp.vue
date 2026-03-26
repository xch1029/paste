<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { copyItem, getHistory, hideCurrentWindow, syncPickerLayout } from "./lib/commands";
import type { ClipboardItem } from "./types";

const items = ref<ClipboardItem[]>([]);
const loading = ref(true);
const errorMessage = ref("");
const copiedItemId = ref<number | null>(null);
const trackRef = ref<HTMLElement | null>(null);
const unsubscribers: Array<() => void> = [];

async function resetTrackPosition() {
  await nextTick();
  trackRef.value?.scrollTo({ left: 0, behavior: "auto" });
}

async function refreshHistory() {
  try {
    const response = await getHistory("");
    items.value = response.items.slice(0, 24);
    errorMessage.value = "";
    await syncPickerLayout();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function handleCopy(item: ClipboardItem) {
  if (!item) {
    return;
  }

  copiedItemId.value = item.id;
  await copyItem(item.id);
  window.setTimeout(() => {
    if (copiedItemId.value === item.id) {
      copiedItemId.value = null;
    }
  }, 420);
  await hideCurrentWindow();
}

async function handleHide() {
  await hideCurrentWindow();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void handleHide();
  }
}

function onWindowFocus() {
  void resetTrackPosition();
}

onMounted(async () => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("focus", onWindowFocus);

  const historyListener = await listen("clipboard-history-changed", () => {
    void refreshHistory();
  });

  unsubscribers.push(historyListener);

  await refreshHistory();
  await resetTrackPosition();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("focus", onWindowFocus);
  unsubscribers.forEach((unlisten) => unlisten());
});
</script>

<template>
  <main class="picker-shell">
    <section class="picker-panel">
      <div v-if="errorMessage" class="picker-state">
        <strong>无法加载历史记录</strong>
        <span>{{ errorMessage }}</span>
      </div>
      <div v-else-if="loading" class="picker-state">
        <strong>正在加载</strong>
        <span>正在准备快速列表。</span>
      </div>
      <div v-else-if="items.length === 0" class="picker-state">
        <strong>暂无结果</strong>
        <span>复制一些文本或图片后，这里会出现最近历史。</span>
      </div>

      <section v-else ref="trackRef" class="picker-track">
        <article
          v-for="item in items"
          :key="item.id"
          class="picker-card"
          :class="{ 'picker-card-copied': copiedItemId === item.id }"
          tabindex="0"
          @dblclick="handleCopy(item)"
        >
          <span class="picker-card__type">
            {{ item.itemType === "image" ? "图片" : "文本" }}
          </span>

          <img
            v-if="item.itemType === 'image' && item.imageUrl"
            :src="item.imageUrl"
            :alt="item.preview"
            class="picker-card__image"
          />

          <p class="picker-card__preview">{{ item.preview }}</p>
          <p class="picker-card__meta">
            <template v-if="item.imageSize">
              {{ item.imageSize.width }} × {{ item.imageSize.height }}
            </template>
            <template v-else-if="item.updatedAt">
              {{ new Date(item.updatedAt).toLocaleTimeString() }}
            </template>
            <span v-if="item.isPinned"> · 已置顶</span>
          </p>
        </article>
      </section>
    </section>
  </main>
</template>
