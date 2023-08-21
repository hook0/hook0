<script setup lang="ts">
import debounce from 'lodash.debounce';
import { isString } from 'fp-ts/string';
import { ref, watch } from 'vue';

import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import Hook0Button from '@/components/Hook0Button.vue';

/**
 * Hook0-Key-Value can manipulate either:
 * - an object
 * - an array of key-value pairs object (Hook0KeyValueKeyValuePair)
 */
function getDefaultItem(): Hook0KeyValueKeyValuePair {
  return { key: '', value: '' };
}

type Hook0KeyValuePlainObject = Record<string, string>;

enum RWMode {
  ARRAY,
  OBJECT,
}

interface iRWMode<T> {
  is(val: T): boolean;
  init(val: T): Hook0KeyValueKeyValuePair[];
  write(val: Hook0KeyValueKeyValuePair[]): T;
}

const MODE = {
  [RWMode.ARRAY]: {
    is(val: Hook0KeyValueKeyValuePair[]) {
      return (
        Array.isArray(val) &&
        // eslint-disable-next-line no-prototype-builtins
        val.every((item) => item.hasOwnProperty('key') && item.hasOwnProperty('value'))
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
        Object.entries(val).every(([key, value]) => isString(key) && isString(value))
      );
    },
    init(val: Hook0KeyValuePlainObject) {
      const entries = Object.entries(val);
      return entries.length === 0
        ? [getDefaultItem()]
        : entries.map(([key, value]) => ({ key, value }));
    },
    write(val: Hook0KeyValueKeyValuePair[]): Hook0KeyValuePlainObject {
      return <Hook0KeyValuePlainObject>val.reduce((m, { key, value }) => {
        // @ts-ignore
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

  //  always start with at least one element
  const pairs = encoder.init(val as Hook0KeyValueKeyValuePair[] & Hook0KeyValuePlainObject);
  return { encoder, pairs };
}

interface Props {
  value: Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject;
  keyPlaceholder?: string;
  valuePlaceholder?: string;
}

const props = defineProps<Props>();
const rawEmit = defineEmits(['update:modelValue']);

const state = ref(getNewInternalState(props.value));

watch(
  () => props.value,
  (_newVal, _oldVal) => {
    const newState = getNewInternalState(props.value);
    state.value = newState;
  }
);

function _emit() {
  rawEmit(
    'update:modelValue',
    state.value.encoder.write(state.value.pairs.filter(({ key }) => key.length > 0))
  );
}
const emit = debounce(_emit);

function remove(index: number) {
  state.value.pairs.splice(index, 1);
  emit();
}

function add() {
  state.value.pairs.push(getDefaultItem());
  emit();
}
</script>

<template>
  <div class="w-full">
    <div v-for="(item, index) in state.pairs" :key="index" class="kv-item">
      <Hook0Input
        v-model="item.key"
        type="text"
        class="col-span-4"
        :placeholder="keyPlaceholder"
        @input="emit()"
      ></Hook0Input>
      <Hook0Input
        v-model="item.value"
        type="text"
        class="col-span-4"
        :placeholder="valuePlaceholder"
        @input="emit()"
      ></Hook0Input>
      <Hook0Button
        :disabled="state.pairs.length === 1"
        class="white col-span-1"
        @click="remove(index)"
      >
        <Hook0Icon name="fa-minus"></Hook0Icon>
      </Hook0Button>
      <Hook0Button class="white col-span-1" @click="add()">
        <Hook0Icon name="fa-plus"></Hook0Icon>
      </Hook0Button>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.kv-item {
  @apply grid grid-cols-10 gap-4 mb-4;
}
</style>
