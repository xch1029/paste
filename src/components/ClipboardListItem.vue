<script setup lang="ts">
import type { ClipboardItem } from "../types";

defineProps<{
  item: ClipboardItem;
  active: boolean;
}>();

const emit = defineEmits<{
  select: [];
  copy: [];
  togglePin: [];
  remove: [];
}>();
</script>

<template>
  <article
    class="history-card"
    :class="{ 'history-card-active': active }"
    tabindex="0"
    @click="emit('select')"
    @focus="emit('select')"
    @dblclick="emit('copy')"
  >
    <div class="history-card__type">
      <span class="history-card__badge">{{ item.itemType === "image" ? "图片" : "文本" }}</span>
      <span v-if="item.isPinned" class="history-card__pin">已置顶</span>
    </div>

    <div class="history-card__content">
      <img
        v-if="item.itemType === 'image' && item.imageUrl"
        :src="item.imageUrl"
        :alt="item.preview"
        class="history-card__image"
      />

      <div class="history-card__text">
        <p class="history-card__preview">{{ item.preview }}</p>
        <p class="history-card__meta">
          <template v-if="item.imageSize">
            {{ item.imageSize.width }} × {{ item.imageSize.height }}
          </template>
          <template v-else>
            {{ new Date(item.updatedAt).toLocaleString() }}
          </template>
        </p>
      </div>
    </div>

    <div class="history-card__actions">
      <button class="ghost-button" type="button" @click.stop="emit('copy')">
        复制
      </button>
      <button class="ghost-button" type="button" @click.stop="emit('togglePin')">
        {{ item.isPinned ? "取消置顶" : "置顶" }}
      </button>
      <button class="ghost-button ghost-button-danger" type="button" @click.stop="emit('remove')">
        删除
      </button>
    </div>
  </article>
</template>
