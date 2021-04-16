<template>
  <slot></slot>
</template>

<script lang="ts">

import {PropType} from 'vue';
import { Vue, Options } from 'vue-class-component'


import {UIRouterPlugin, servicesPlugin, PluginFactory, StateDeclaration} from '@uirouter/core';
import {computed} from "@vue/reactivity";
import {UIRouterVue, VueStateDeclaration} from "..";

/** @hidden */
export const InstanceOrPluginsMissingError = `Router instance or plugins missing.
 You must either provide a location plugin via the plugins <UIRouter plugins={[pushStateLocationPlugin]} states={[···]}>
   <UIView />
 </UIRouter>

 or initialize the router yourself and pass the instance via const router = new UIRouterVue();
 router.plugin(pushStateLocationPlugin);
 ···
 <UIRouter router={router}>
   <UIView />
 </UIRouter>
 `;

/**
 * This Vue Context component lets you access the UIRouter instance anywhere in the component tree
 *
 * When using hooks, use [[useRouter]] instead.
 *
 * #### Example:
 * ```jsx
 * <UIRouterContext.Consumer>
 *  {router => <MyComponent router={router} />}
 * </UIRouterContext.Consumer>
 * ```
 */
@Options<UIRouter>({
  props: {
    /**
     * The root application content.
     * Typically this will render a [[UIView]] viewport component */
    children: {
      type: Object as PropType<any>,
      required: false
    },
    /** UIRouter Plugins (used with "Component Setup" bootstrapping, see [[UIRouter]]) */
    plugins: {type: Array as PropType<Array<PluginFactory<UIRouterPlugin>>>, required: false},
    /** The application states (used with "Component Setup" bootstrapping, see [[UIRouter]]) */
    states: {type: Array as PropType<Array<VueStateDeclaration>>, required: false},
    /**
     * A callback function to do imperative configuration after the [[UIRouterVue]] object is created
     * (used with "Component Setup" bootstrapping, see [[UIRouter]])
     */
    config: {type: Function as PropType<(router: UIRouterVue) => void>, required: false},
    /** The pre-configured UIRouterVue instance (used with "Manual Setup", see [[UIRouter]]) */
    router: {type: Object as PropType<UIRouterVue>, required: false}
  },

  provide() {
    return {
      $router: computed(() => this.current)
    }
  },

  mounted() {
    const {router, config, plugins, states} = this.$props;
    if (router && !plugins && !states) {
      this.current = router;
    } else if (plugins && states) {
      // We need to create a new instance of the Router and register plugins, config and states
      this.current = new UIRouterVue();
      this.current.plugin(servicesPlugin); // services plugins is necessary for the router to function
      plugins.forEach((plugin : any) => this.current.plugin(plugin));

      if (config) {
        config(this.current);
      }

      (states || []).forEach((state: StateDeclaration) => this.current.stateRegistry.register(state));
    } else {
      throw new Error(InstanceOrPluginsMissingError);
    }

    this.current.start();
  }
})
export default class UIRouter extends Vue {
  current!: UIRouterVue
};
</script>
