/**
 * # Vue Specific API
 *
 * UI-Router for Vue relies heavily on [`@uirouter/core`](http://github.com/ui-router/core).
 * The following APIs are extensions to the core ui-router APIs, specific to `@uirouter/vue`.
 */

// We need to export everything *BUT* UIRouter
import { App } from 'vue';
import { UIRouter, UIView } from './components';
import { UIRouterVue } from './core';
import { Vue } from 'vue-class-component';

export { ParamDeclaration, StateDeclaration } from '@uirouter/core';
export * from './interface';
export * from './vueViews';
export * from './components';

export { UIRouterVue, StartMethodCalledMoreThanOnceError } from './core';

export let _Vue: App;

export function install(app: App, ...options: any[]) {
  // @ts-ignore
  if (install.installed && _Vue === app) {
    return;
  }

  _Vue = app;

  // @ts-ignore
  install.installed = true;

  const isDef = (v: any) => v !== undefined;

  /*const registerInstance = (vm, callVal) => {
    let i = vm.$options._parentVnode;
    if (isDef(i) && isDef((i = i.data)) && isDef((i = i.registerRouteInstance))) {
      i(vm, callVal);
    }
  };*/

  class UIRouterUnsupportError extends Error {}

  app.mixin({
    beforeCreate() {
      console.log('ok', this.$options.router);
      if (isDef(this.$options.router)) {
        debugger;
        this._routerRoot = this;
        this._router = this.$options.router;
        this._router.init(this);
        //reactive(this, '_route', this._router.history.current);
      } else {
        this._routerRoot = (this.$parent && this.$parent._routerRoot) || this;
      }
      //registerInstance(this, this);
    },
    unmounted() {
      //registerInstance(this);
    },
  });
  /*
  debugger;
  // @ts-ignore
  Object.defineProperty(app.prototype, '$router', {
    get() {
      return this._routerRoot._router;
    },
  });

  // @ts-ignore
  Object.defineProperty(app.prototype, '$route', {
    get() {
      return this._routerRoot._route;
    },
  });*/

  app.component('UIView', UIView);
  app.component('UIRouter', UIRouter);
}
