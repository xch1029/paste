<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import ClipboardListItem from "./components/ClipboardListItem.vue";
import {
  clearHistory,
  copyItem,
  deleteItem,
  getAppState,
  getHistory,
  hidePanel,
  setMonitoringPaused,
  toggleItemPin,
} from "./lib/commands";
import type { AppStatus, ClipboardItem } from "./types";

const search = ref("");
const items = ref<ClipboardItem[]>([]);
const status = ref<AppStatus>({
  monitoringPaused: false,
  historyCount: 0,
  hotkey: "Ctrl+Shift+V",
});
const activeIndex = ref(0);
const loading = ref(true);
const errorMessage = ref("");
const lastAction = ref("正在等待你的下一次复制。");
const searchInput = ref<HTMLInputElement | null>(null);

let refreshTimer: number | null = null;
const unsubscribers: Array<() => void> = [];

const selectedItem = computed(() => items.value[activeIndex.value] ?? null);

function focusSearch() {
  searchInput.value?.focus();
  searchInput.value?.select();
}

function ensureActiveIndex() {
  if (items.value.length === 0) {
    activeIndex.value = 0;
    return;
  }
  activeIndex.value = Math.min(activeIndex.value, items.value.length - 1);
}

async function refreshHistory() {
  try {
    const response = await getHistory(search.value);
    items.value = response.items;
    ensureActiveIndex();
    errorMessage.value = "";
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function refreshStatus() {
  try {
    status.value = await getAppState();
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function refreshAll() {
  await Promise.all([refreshHistory(), refreshStatus()]);
}

async function handleCopy(item = selectedItem.value) {
  if (!item) {
    return;
  }
  await copyItem(item.id);
  lastAction.value = `已将${item.itemType === "image" ? "图片" : "文本"}重新写回系统剪贴板。`;
  await refreshAll();
}

async function handleTogglePin(item: ClipboardItem) {
  await toggleItemPin(item.id);
  lastAction.value = item.isPinned ? "已取消置顶。" : "已置顶到顶部。";
  await refreshAll();
}

async function handleDelete(item: ClipboardItem) {
  await deleteItem(item.id);
  lastAction.value = "已从剪贴板历史中删除该条目。";
  await refreshAll();
}

async function handlePauseToggle() {
  const next = !status.value.monitoringPaused;
  status.value = await setMonitoringPaused(next);
  lastAction.value = next
    ? "已暂停剪贴板监听。"
    : "已恢复剪贴板监听。";
}

async function handleClearHistory() {
  await clearHistory();
  lastAction.value = "已清空全部剪贴板历史。";
  await refreshAll();
}

async function handleHide() {
  await hidePanel();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowDown") {
    event.preventDefault();
    activeIndex.value = Math.min(activeIndex.value + 1, Math.max(items.value.length - 1, 0));
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    activeIndex.value = Math.max(activeIndex.value - 1, 0);
    return;
  }

  if (event.key === "Enter" && selectedItem.value) {
    event.preventDefault();
    void handleCopy();
    return;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    void handleHide();
  }
}

watch(search, () => {
  if (refreshTimer) {
    window.clearTimeout(refreshTimer);
  }
  refreshTimer = window.setTimeout(() => {
    void refreshHistory();
  }, 150);
});

onMounted(async () => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("focus", focusSearch);

  const historyListener = await listen("clipboard-history-changed", () => {
    void refreshAll();
  });
  const stateListener = await listen("clipboard-state-changed", () => {
    void refreshStatus();
  });

  unsubscribers.push(historyListener, stateListener);

  await refreshAll();
  focusSearch();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("focus", focusSearch);
  if (refreshTimer) {
    window.clearTimeout(refreshTimer);
  }
  unsubscribers.forEach((unlisten) => unlisten());
});
</script>

<template>
  <main class="shell">
    <section class="panel">
      <div class="layout">
        <aside class="sidebar">
          <div>
            <p class="eyebrow">剪贴板档案</p>
            <h1 class="headline">paste</h1>
            <p class="lede">
              一个以键盘操作为核心的剪贴板面板，支持文本与图片。你可以快速搜索、置顶、找回，并一键把内容重新送回系统剪贴板。
            </p>
          </div>

          <div class="metric-grid">
            <div class="metric-card">
              <span class="metric-card__label">历史数量</span>
              <strong class="metric-card__value">{{ status.historyCount }}</strong>
            </div>
            <div class="metric-card">
              <span class="metric-card__label">全局快捷键</span>
              <strong class="metric-card__value metric-card__value-small">
                {{ status.hotkey }}
              </strong>
            </div>
            <div class="metric-card">
              <span class="metric-card__label">最近操作</span>
              <strong class="metric-card__value metric-card__value-small">
                {{ lastAction }}
              </strong>
            </div>
          </div>

          <div class="action-stack">
            <button class="action-button action-button-accent" type="button" @click="handlePauseToggle">
              {{ status.monitoringPaused ? "恢复监听" : "暂停监听" }}
            </button>
            <button class="action-button action-button-muted" type="button" @click="handleClearHistory">
              清空历史
            </button>
            <button class="action-button action-button-muted" type="button" @click="handleHide">
              隐藏面板
            </button>
          </div>

          <div class="legend">
            <div class="legend__row">
              <span>方向键</span>
              <span>移动选中项</span>
            </div>
            <div class="legend__row">
              <span>回车</span>
              <span>复制当前项</span>
            </div>
            <div class="legend__row">
              <span>Esc</span>
              <span>隐藏窗口</span>
            </div>
          </div>
        </aside>

        <section class="content">
          <header class="toolbar">
            <input
              ref="searchInput"
              v-model="search"
              class="search-input"
              type="search"
              placeholder="搜索剪贴板历史..."
            />

            <div class="status-row">
              <span
                class="status-pill"
                :class="status.monitoringPaused ? 'status-pill-paused' : 'status-pill-live'"
              >
                {{ status.monitoringPaused ? "监听已暂停" : "正在监听" }}
              </span>
              <span class="status-pill">当前显示 {{ items.length }} 条</span>
            </div>
          </header>

          <section class="board">
            <header class="board__header">
              <div>
                <h2 class="board__title">剪贴板回收站</h2>
                <p class="board__subtitle">
                  置顶常用内容，浏览最近复制记录，并用一次按键把任意内容重新激活。
                </p>
              </div>
            </header>

            <div v-if="errorMessage" class="error-state">
              <div>
                <strong>无法加载剪贴板数据</strong>
                <span>{{ errorMessage }}</span>
              </div>
            </div>

            <div v-else-if="loading" class="empty-state">
              <div>
                <strong>正在加载剪贴板历史</strong>
                <span>正在准备你的本地档案。</span>
              </div>
            </div>

            <div v-else-if="items.length === 0" class="empty-state">
              <div>
                <strong>暂时没有匹配结果</strong>
                <span>
                  先复制一些文本或图片来建立历史记录，或者清空当前搜索条件。
                </span>
              </div>
            </div>

            <div v-else class="board__list">
              <ClipboardListItem
                v-for="(item, index) in items"
                :key="item.id"
                :item="item"
                :active="index === activeIndex"
                @select="activeIndex = index"
                @copy="handleCopy(item)"
                @toggle-pin="handleTogglePin(item)"
                @remove="handleDelete(item)"
              />
            </div>
          </section>
        </section>
      </div>
    </section>
  </main>
</template>
