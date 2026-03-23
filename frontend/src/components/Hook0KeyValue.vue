<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { Minus, Plus } from 'lucide-vue-next';

const { t } = useI18n();

let pairIdCounter = 0;

type InternalPair = Hook0KeyValueKeyValuePair & {
  _id: number;
};

function getDefaultItem(): InternalPair {
  return { key: '', value: '', _id: pairIdCounter++ };
}

type Hook0KeyValuePlainObject = Record<string, string>;

enum RWMode {
  ARRAY,
  OBJECT,
}

type iRWMode<T> = {
  is(val: T): boolean;
  init(val: T): Hook0KeyValueKeyValuePair[];
  write(val: Hook0KeyValueKeyValuePair[]): T;
};

const MODE = {
  [RWMode.ARRAY]: {
    is(val: Hook0KeyValueKeyValuePair[]) {
      return (
        Array.isArray(val) &&
        val.every(
          (item) =>
            Object.prototype.hasOwnProperty.call(item, 'key') &&
            Object.prototype.hasOwnProperty.call(item, 'value')
        )
      );
    },
    init(val: Hook0KeyValueKeyValuePair[]) {
      return val.length === 0 ? [getDefaultItem()] : val;
    },
    write(val: Hook0KeyValueKeyValuePair[]): Hook0KeyValueKeyValuePair[] {
      return val;
    },
  } as iRWMode<Hook0KeyValueKeyValuePair[]>,
  [RWMode.OBJECT]: {
    is(val: Hook0KeyValuePlainObject) {
      return (
        typeof val === 'object' &&
        Object.entries(val).every(
          ([key, value]) => typeof key === 'string' && typeof value === 'string'
        )
      );
    },
    init(val: Hook0KeyValuePlainObject) {
      const entries = Object.entries(val);
      return entries.length === 0
        ? [getDefaultItem()]
        : entries.map(([key, value]) => ({ key, value }));
    },
    write(val: Hook0KeyValueKeyValuePair[]): Hook0KeyValuePlainObject {
      return val.reduce<Hook0KeyValuePlainObject>((m, { key, value }) => {
        m[key] = value;
        return m;
      }, {});
    },
  } as iRWMode<Hook0KeyValuePlainObject>,
};

function getNewInternalState(val: Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject) {
  const encoder = MODE[RWMode.ARRAY].is(val as Hook0KeyValueKeyValuePair[])
    ? MODE[RWMode.ARRAY]
    : MODE[RWMode.OBJECT];

  const raw = encoder.init(val as Hook0KeyValueKeyValuePair[] & Hook0KeyValuePlainObject);
  const pairs: InternalPair[] = raw.map((p) =>
    '_id' in p ? (p as InternalPair) : { ...p, _id: pairIdCounter++ }
  );
  return { encoder, pairs };
}

type Props = {
  value: Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject;
  keyPlaceholder?: string;
  valuePlaceholder?: string;
};

const props = withDefaults(defineProps<Props>(), {
  keyPlaceholder: undefined,
  valuePlaceholder: undefined,
});
const rawEmit = defineEmits<{
  'update:modelValue': [value: Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject];
}>();

const state = ref(getNewInternalState(props.value));

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.value,
  () => {
    state.value = getNewInternalState(props.value);
  }
);

function emitUpdate() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    rawEmit(
      'update:modelValue',
      state.value.encoder.write(state.value.pairs.filter(({ key }) => key.length > 0))
    );
  }, 150);
}

function remove(index: number) {
  state.value.pairs.splice(index, 1);
  emitUpdate();
}

function add() {
  state.value.pairs.push(getDefaultItem());
  emitUpdate();
}
</script>

<template>
  <div class="w-full">
    <div
      v-for="(item, index) in state.pairs"
      :key="item._id"
      class="kv-item"
      :data-test="`kv-item-${index}`"
    >
      <Hook0Input
        v-model="item.key"
        type="text"
        class="col-span-4"
        :placeholder="keyPlaceholder ?? t('common.key')"
        :data-test="`kv-key-input-${index}`"
        @input="emitUpdate()"
      />
      <Hook0Input
        v-model="item.value"
        type="text"
        class="col-span-4"
        :placeholder="valuePlaceholder ?? t('common.value')"
        :data-test="`kv-value-input-${index}`"
        @input="emitUpdate()"
      />
      <Hook0Button
        :disabled="state.pairs.length === 1"
        variant="secondary"
        size="sm"
        class="col-span-1"
        :aria-label="t('common.remove')"
        :data-test="`kv-remove-button-${index}`"
        @click="remove(index)"
      >
        <Minus :size="16" aria-hidden="true" />
      </Hook0Button>
      <Hook0Button
        variant="secondary"
        size="sm"
        class="col-span-1"
        :aria-label="t('common.add')"
        :data-test="`kv-add-button-${index}`"
        @click="add()"
      >
        <Plus :size="16" aria-hidden="true" />
      </Hook0Button>
    </div>
  </div>
</template>

<style scoped>
.kv-item {
  display: flex;
  align-items: stretch;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.col-span-4 {
  flex: 1;
  min-width: 0;
}

.col-span-1 {
  flex-shrink: 0;
}

/* Make buttons match input height */
.col-span-1 :deep(.hook0-button) {
  height: 100%;
}
</style>
