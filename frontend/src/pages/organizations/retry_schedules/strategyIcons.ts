import { defineComponent, h } from 'vue';

// Sparkline icons for the retry-schedule strategy picker. Each one hints at the
// shape of delay over retry #: exponential J-curve, flat line, or irregular zigzag.

const baseSvgProps = (size: number) => ({
  width: size,
  height: size,
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': 2,
  'stroke-linecap': 'round',
  'stroke-linejoin': 'round',
  'aria-hidden': 'true',
});

export const ExponentialStrategyIcon = defineComponent({
  name: 'ExponentialStrategyIcon',
  props: { size: { type: Number, default: 18 } },
  setup: (props) => () =>
    h('svg', baseSvgProps(props.size), [
      h('polyline', { points: '3,21 8,20 12,17 16,11 20,3' }),
      h('circle', { cx: 3, cy: 21, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 20, cy: 3, r: 1, fill: 'currentColor' }),
    ]),
});

export const LinearStrategyIcon = defineComponent({
  name: 'LinearStrategyIcon',
  props: { size: { type: Number, default: 18 } },
  setup: (props) => () =>
    h('svg', baseSvgProps(props.size), [
      h('polyline', { points: '3,12 21,12' }),
      h('circle', { cx: 3, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 8, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 12, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 16, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 21, cy: 12, r: 1.3, fill: 'currentColor' }),
    ]),
});

export const CustomStrategyIcon = defineComponent({
  name: 'CustomStrategyIcon',
  props: { size: { type: Number, default: 18 } },
  setup: (props) => () =>
    h('svg', baseSvgProps(props.size), [
      h('polyline', { points: '3,16 8,8 12,14 16,5 20,11' }),
      h('circle', { cx: 3, cy: 16, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 8, cy: 8, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 12, cy: 14, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 16, cy: 5, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 20, cy: 11, r: 1, fill: 'currentColor' }),
    ]),
});
