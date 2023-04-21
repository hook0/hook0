<template>
  <div class="w-full">
    <div class="kv-item" v-for="(item, index) in pairs" :key="index">
      <hook0-input
        type="text"
        @input="emit()"
        v-model="item.key"
        class="col-span-4"
        :placeholder="keyPlaceholder"
      ></hook0-input>
      <hook0-input
        type="text"
        @input="emit()"
        v-model="item.value"
        class="col-span-4"
        :placeholder="valuePlaceholder"
      ></hook0-input>
      <hook0-button @click="remove(index)" :disabled="pairs.length === 1" class="white col-span-1">
        <hook0-icon name="fa-minus"></hook0-icon>
      </hook0-button>
      <hook0-button @click="add(index)" class="white col-span-1">
        <hook0-icon name="fa-plus"></hook0-icon>
      </hook0-button>
    </div>
  </div>
</template>

<script lang="ts">
import { Options, Vue } from 'vue-class-component';
import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import debounce from 'lodash.debounce';
import { DebouncedFuncLeading } from 'lodash';
import { isString } from 'fp-ts/string';
import { defineComponent } from 'vue';

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

export default defineComponent({
  name: 'hook0-key-value',
  props: {
    /**
     * note that this value will be mutated
     */
    value: {
      type: Object,
      required: true,
      validator: (val: Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject) =>
        MODE[RWMode.ARRAY].is(val as Hook0KeyValueKeyValuePair[]) ||
        MODE[RWMode.OBJECT].is(val as Hook0KeyValuePlainObject),
    },
    keyPlaceholder: {
      type: String,
      required: false,
      default: '',
    },
    valuePlaceholder: {
      type: String,
      required: false,
      default: '',
    },
  },
  data() {
    const { encoder, pairs } = getNewInternalState(
      this.value as Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject
    );

    return {
      encoder,
      pairs,
      emit: this._emit,
    };
  },
  computed: {},

  watch: {
    value(newVal, oldVal) {
      const { encoder, pairs } = getNewInternalState(
        newVal as Hook0KeyValueKeyValuePair[] | Hook0KeyValuePlainObject
      );
      this.encoder = encoder;
      this.pairs = pairs;
    },
  },

  mounted() {
    this._internalState();
  },

  beforeUpdate() {
    this._internalState();
  },

  methods: {
    _internalState() {
      this.emit = debounce(this._emit.bind(this));
    },

    _emit() {
      this.$emit('update:modelValue', this.encoder.write(this.pairs));
    },
    /**
     *
     * @param {Number} index
     */
    remove(index: number) {
      this.pairs.splice(index, 1);
      this.emit();
    },

    /**
     *
     */
    add() {
      this.pairs.push(getDefaultItem());
      this.emit();
    },
  },
});
</script>

<style lang="scss" scoped>
.kv-item {
  @apply grid grid-cols-10 gap-4 mb-4;
}
</style>
