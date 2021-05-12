<template>
  <div class="hook0-pagination-container">
    <div>
      <i18n path="pagination.info" tag="p" class="hook0-pagination-info">
        <template v-slot:page>
          <hook0-text class="medium">{{ pageCurrent }}</hook0-text>
        </template>
        <template v-slot:pages>
          <hook0-text class="medium">{{ pageCount }}</hook0-text>
        </template>
        <template v-slot:results>
          <hook0-text class="medium">{{ resultCount }}</hook0-text>
        </template>
      </i18n>
    </div>
    <div>
      <nav class="hook0-pagination-pages" aria-label="Pagination">
        <hook0-button class="white left" :disabled="pageCurrent === 1" @click="$emit('change', pageCurrent - 1)">
          <span class="sr-only">{{ $t('pagination.previous') }}</span>
          <hook0-icon name="chevron-left"></hook0-icon>
        </hook0-button>
        <hook0-button
          class="white right"
          :disabled="pageCurrent === pageCount"
          @click="$emit('change', pageCurrent + 1)"
        >
          <span class="sr-only">{{ $t('pagination.next') }}</span>
          <hook0-icon name="chevron-right"></hook0-icon>
        </hook0-button>
      </nav>
    </div>
  </div>
</template>

<script>
import Hook0Text from '@/components/Hook0Text';
import Hook0Icon from '@/components/Hook0Icon';
import Hook0Button from '@/components/Hook0Button';

export default {
  name: 'hook0-pagination',
  components: {
    Hook0Button,
    Hook0Icon,
    Hook0Text,
  },
  data() {
    return {};
  },
  props: {
    displayedSlice: {
      type: Number,
      default: 5,
    },
    pageCurrent: {
      type: Number,
      required: true,
    },
    pageCount: {
      type: Number,
      required: true,
    },
    resultCount: {
      type: Number,
      required: true,
    },
  },
  methods: {},
  mounted() {},
  computed: {},
};
</script>

<style lang="scss" scoped>
.hook0-pagination-container {
  @apply hidden sm:flex-1 sm:flex sm:items-center sm:justify-between;
}

.hook0-pagination-info {
  @apply text-sm text-gray-700;
}

.hook0-pagination-pages {
  @apply relative z-0 inline-flex -space-x-px;
}
</style>
