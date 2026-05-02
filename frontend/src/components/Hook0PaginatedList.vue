<script setup lang="ts" generic="T">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Spinner from '@/components/Hook0Spinner.vue';
import {
  isNextDisabled,
  isPrevDisabled,
  pageIndicator,
} from '@/components/Hook0PaginatedList.logic';
import type { UseCursorInfiniteQueryResult } from '@/composables/useCursorInfiniteQuery';

const { t } = useI18n();

const props = defineProps<{
  query: UseCursorInfiniteQueryResult<T>;
}>();

defineSlots<{
  default(props: { items: T[] }): unknown;
}>();

// Pages are 1-indexed for display.
const displayPage = computed(() =>
  pageIndicator({ currentPageIdx: props.query.currentPageIdx.value })
);
const items = computed<T[]>(() => props.query.currentPageItems.value);
const isLoading = computed(() => props.query.isLoading.value);
const prevDisabled = computed(() =>
  isPrevDisabled({
    isLoading: isLoading.value,
    hasPreviousPage: props.query.hasPreviousPage.value,
  })
);
const nextDisabled = computed(() =>
  isNextDisabled({ isLoading: isLoading.value, hasNextPage: props.query.hasNextPage.value })
);

function goPrev(): void {
  void props.query.fetchPreviousPage();
}

function goNext(): void {
  void props.query.fetchNextPage();
}
</script>

<template>
  <div class="hook0-paginated-list">
    <div class="hook0-paginated-list__rows">
      <slot :items="items" />
    </div>

    <nav
      class="hook0-paginated-list__controls"
      role="navigation"
      :aria-label="t('pagination.previous') + ' / ' + t('pagination.next')"
    >
      <Hook0Button
        variant="ghost"
        size="sm"
        :disabled="prevDisabled"
        :aria-label="t('pagination.previous')"
        data-test="pagination-prev"
        @click="goPrev"
      >
        <template #left>
          <ChevronLeft :size="16" aria-hidden="true" />
        </template>
        <span class="hook0-paginated-list__btn-label">{{ t('pagination.previous') }}</span>
      </Hook0Button>

      <span
        class="hook0-paginated-list__indicator"
        data-test="pagination-current-page"
        aria-live="polite"
      >
        {{ t('pagination.currentPage', { page: displayPage }) }}
      </span>

      <Hook0Button
        variant="ghost"
        size="sm"
        :disabled="nextDisabled"
        :aria-label="t('pagination.next')"
        data-test="pagination-next"
        @click="goNext"
      >
        <span class="hook0-paginated-list__btn-label">{{ t('pagination.next') }}</span>
        <template #right>
          <ChevronRight :size="16" aria-hidden="true" />
        </template>
      </Hook0Button>

      <span v-if="isLoading" class="hook0-paginated-list__loading" aria-live="polite">
        <Hook0Spinner :size="14" />
        <span class="hook0-paginated-list__loading-label">{{ t('pagination.loading') }}</span>
      </span>
    </nav>
  </div>
</template>

<style scoped>
.hook0-paginated-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.hook0-paginated-list__rows {
  display: flex;
  flex-direction: column;
}

.hook0-paginated-list__controls {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.5rem 0.25rem 0;
}

.hook0-paginated-list__indicator {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  padding: 0 0.5rem;
  min-width: 5rem;
  text-align: center;
}

.hook0-paginated-list__loading {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
  margin-left: 0.5rem;
}

.hook0-paginated-list__loading-label {
  font-weight: 500;
}

@media (max-width: 480px) {
  .hook0-paginated-list__btn-label {
    display: none;
  }
}
</style>
