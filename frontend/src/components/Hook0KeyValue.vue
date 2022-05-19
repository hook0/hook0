<template>
  <div class="w-full">
    <div class="kv-item" v-for="(item, index) in pairs" :key="index">
      <hook0-input type="text" @input="emit()" v-model="item.key" class="col-span-4"
                   :placeholder="keyPlaceholder"></hook0-input>
      <hook0-input type="text" @input="emit()" v-model="item.value" class="col-span-4"
                   :placeholder="valuePlaceholder"></hook0-input>
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
import {Options, Vue} from "vue-class-component";
import {Hook0KeyValueKeyValuePair} from "@/components/Hook0KeyValue";
import debounce from "lodash.debounce";
import {DebouncedFuncLeading} from "lodash";

function getDefaultItem(): Hook0KeyValueKeyValuePair {
  return {key: '', value: ''};
}


@Options({
  name: 'hook0-key-value',
  props: {
    /**
     * note that this value will be mutated
     */
    value: {
      type: Array,
      required: true,
      validator: (val: Hook0KeyValueKeyValuePair[]) => {
        // eslint-disable-next-line no-prototype-builtins
        return val.every((item) => item.hasOwnProperty('key') && item.hasOwnProperty('value'));
      },
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
    //  always start with at least one element
    const pairs = (this.value as Hook0KeyValueKeyValuePair[]).length === 0 ? [getDefaultItem()] : this.value;
    return {
      pairs
    };
  },
  computed: {},
})
export default class Hook0KeyValue extends Vue {
  private pairs!: Hook0KeyValueKeyValuePair[];
  private emit!: DebouncedFuncLeading<() => void>;

  mounted() {
    this.emit = debounce(this._emit.bind(this));
  }

  _emit() {
    this.$emit('update:modelValue', this.pairs);
  }

  /**
   *
   * @param {Number} index
   */
  remove(index: number) {
    this.pairs.splice(index, 1);
  }

  /**
   *
   */
  add() {
    this.pairs.push(getDefaultItem());
  }
}
</script>

<style lang="scss" scoped>
.kv-item {
  @apply grid grid-cols-10 gap-4 mb-4;
}
</style>
