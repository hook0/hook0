import { UIRouter, PathNode } from '@uirouter/core';
import { VueViewDeclaration } from './interface';
import { VueViewConfig, vueViewsBuilder } from './vueViews';
import { ViewConfigFactory } from '@uirouter/core/lib/view/view';

/**
 * Vue View Config Factory
 *
 * Given a path and a [[VueViewDeclaration]]
 * (the view declaration object found on the state declaration),
 * returns a [[VueViewConfig]]
 *
 * The VueViewConfig is an instance of a view,
 * which will be provided to the matching `UIView` Component's
 * [[UIView.viewConfigUpdated]] function.
 *
 * @internal
 */
let viewConfigFactory: ViewConfigFactory = (node, config) => new VueViewConfig(node as [PathNode], config);

/** @hidden */
export const StartMethodCalledMoreThanOnceError = `
  The Router.start() method has been called more than once.
  The <UIRouter> component calls start() as final step of the initialization and you shouldn't need to call it manually.
`;

/**
 * The main UIRouter object
 *
 * This is the main UIRouter object.
 * There should be one instance of this object per running application.
 *
 * This class has references to all the other UIRouter services.
 */
export class UIRouterVue extends UIRouter {
  started = false;
  /**
   * Creates a new UIRouter instance
   *
   * This can be used to manually bootstrap the router.
   *
   * #### Example:
   * ```js
   * import { UIRouterVue } from "ui-router-vue";
   * let routerInstance = new UIRouterVue();
   * routerInstance.start();
   * ```
   */
  constructor() {
    super();
    this.viewService._pluginapi._viewConfigFactory('vue', viewConfigFactory);
    this.stateRegistry.decorator('views', vueViewsBuilder);
  }

  /**
   * Starts the router
   *
   * Calling this method enables listening to the URL for changes.
   * It also performs the initial state synchronization from the URL.
   */
  start(): void {
    // Throw error if user calls `start` more than once
    if (this.started) {
      throw new Error(StartMethodCalledMoreThanOnceError);
    } else {
      // Starts or stops listening for URL changes
      // Call this sometime after calling [[deferIntercept]] to start monitoring the url. This causes UI-Router to start listening for changes to the URL, if it wasn't already listening.
      this.urlService.listen(true);

      // Activates the best rule for the current URL
      // Checks the current URL for a matching [[UrlRule]], then invokes that rule's handler. This method is called internally any time the URL has changed.
      // This effectively activates the state (or redirect, etc) which matches the current URL
      this.urlService.sync();

      this.started = true;
    }
  }
}
