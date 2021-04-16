/** @packageDocumentation  @vueapi @module vue */
import { services, forEach, map, pick, PathNode, ViewConfig, ViewService, StateObject } from '@uirouter/core';
import { VueViewDeclaration } from './interface';

/**
 * This is a [[StateBuilder.builder]] function for vue `views`.
 *
 * When the [[StateBuilder]] builds a [[State]] object from a raw [[StateDeclaration]], this builder
 * handles the `views` property with logic specific to ui-router-vue.
 *
 * If no `views: {}` property exists on the [[StateDeclaration]], then it creates the `views` object and
 * applies the state-level configuration to a view named `$default`.
 *
 * @internal
 */
export function vueViewsBuilder(state: StateObject) {
  let views: any = {},
    viewsDefinitionObject;
  if (!state.views) {
    viewsDefinitionObject = { $default: pick(state, ['component']) };
  } else {
    viewsDefinitionObject = map(state.views, (val: any, key) => {
      if (val.component) return val;
      return { component: val };
    });
  }

  forEach(viewsDefinitionObject, function (config: any, name: string) {
    name = name || '$default'; // Account for views: { "": { template... } }
    if (Object.keys(config).length == 0) return;

    config.$type = 'vue';
    config.$context = state;
    config.$name = name;

    let normalized = ViewService.normalizeUIViewTarget(config.$context, config.$name);
    config.$uiViewName = normalized.uiViewName;
    config.$uiViewContextAnchor = normalized.uiViewContextAnchor;

    views[name] = config;
  });
  return views;
}

/** @internal */
let id = 0;

/** @internal */
export class VueViewConfig implements ViewConfig {
  loaded: boolean = true;
  $id: number = id++;

  constructor(public path: [PathNode], public viewDecl: VueViewDeclaration) {}

  load() {
    return services.$q.when(this);
  }
}
