import {VNode} from 'vue';
import {h} from 'vue';
import {watch} from 'vue';


import {
  ResolveContext,
  StateParams,
  Transition,
  TypedMap,
  UIInjector,
  ViewConfig,
  ViewContext,
  applyPairs,
  UIRouter,
} from '@uirouter/core';
//import {useRouter} from '../hooks/useRouter';
import {VueViewConfig} from '../vueViews';
import {VueComponent} from "../interface";
import {VueWithProps} from "vue-class-component";
import { VNodeProps } from 'vue';
import { Ref } from 'vue';
import { isVNode } from 'vue';
import { cloneVNode } from 'vue';
import { createVNode } from 'vue';
import { ref } from 'vue';

/** @internal */
let viewIdCounter = 0;

/** @internal */
/*export interface UIViewAddress {
  context: ViewContext;
  fqn: string;
}*/

/**
 * Interface for [[InjectedProps.resolves]]
 *
 * This Typescript interface shows what fields are available on the `resolves` field.
 */
export interface UIViewResolves {
  /**
   * Any key/value pair defined by a state's resolve
   *
   * If a state defines any [[VueStateDeclaration.resolve]]s, they will be found on this object.
   */
  [key: string]: any;

  /**
   * The `StateParams` for the `Transition` that activated the component
   *
   * This is an alias for:
   * ```js
   * let $stateParams = $transition$.params("to");
   * ```
   */
  $stateParams: StateParams;
  /** The `Transition` that activated the component */
  $transition$: Transition;
}

export interface UIViewInjectedProps {
  transition?: Transition;
  resolves?: UIViewResolves;
  className?: string;
  style?: Object;
}

/** Vue Props for the [[UIView]] component */
export interface UIViewProps {
  /** default content that will be rendered when no child component is loaded into the UIView viewport */
  children?: VueComponent;
  /**
   * The name of the [[UIView]].
   *
   * Assigns a name to this [[UIView]] Portal.
   * see: [Multiple Named Views](https://ui-router.github.io/guide/views#multiple-named-uiviews)
   */
  name?: string;
  /** This prop will be applied to the routed component. */
  className?: string;
  /** This prop will be applied to the routed component. */
  style?: Object;
  /** This render prop can be used to customize the rendering of  routed components.
   *  If the `render` function prop is provided, the `UIView` will use it instead of rendering the component by itself. */
  render?: Function;
}

export const TransitionPropCollisionError =
  '`transition` cannot be used as resolve token. ' +
  'Please rename your resolve to avoid conflicts with the router transition.';

/** @internal */
//export const UIViewContext = createContext<UIViewAddress>(undefined);
/** @deprecated use [[useParentView]] or Vue.useContext(UIViewContext) */
//export const UIViewConsumer = UIViewContext.Consumer;

/** @hidden */
function useResolvesWithStringTokens(resolveContext: ResolveContext, injector: UIInjector) {
  // useMemo
  if (resolveContext && injector) {
    const stringTokens: string[] = resolveContext.getTokens().filter((x) => typeof x === 'string');
    if (stringTokens.indexOf('transition') !== -1) {
      throw new Error(TransitionPropCollisionError);
    }
    return stringTokens.map((token) => [token, injector.get(token)]).reduce(applyPairs, {});
  } else {
    return {};
  }
}

/* @hidden These are the props are passed to the routed component. */
function useRoutedComponentProps(
  router: UIRouter,
  stateName: string,
  viewConfig: Ref<ViewConfig>,
  component: VueComponent,
  resolves: TypedMap<any> | {},
  className: string,
  style: Object,
  transition: any
): UIViewInjectedProps & { key: string } {
  const keyCounterRef = ref(0);

  // useMemo(
  const key = (++keyCounterRef.value).toString();

  // useMemo(
  const baseChildProps = {
    // spread each string resolve as a separate prop
    ...resolves,
    // if a className prop was passed to the UIView, forward it
    className,
    // if a style prop was passed to the UIView, forward it
    style,
    // the transition
    transition,
    // this key updates whenever the state is reloaded, causing the component to remount
    key,
  };

  const maybeRefProp = useUiCanExitClassComponentHook(router, stateName, component);

  // useMemo(
  return {...baseChildProps, ...maybeRefProp};
}

function useViewConfig(){
  const viewConfig = ref<VueViewConfig>();
  const configUpdated = (newConfig: ViewConfig) => {
    if (newConfig !== viewConfig.value) {
      viewConfig.value = (newConfig as VueViewConfig);
    }
  };
  return {viewConfig, configUpdated};
}

/**
 * If a class component is being rendered, wire up its uiCanExit method
 * Return a { ref: Ref<ClassComponentInstance> } if passed a component class
 * Return an empty object {} if passed anything else
 * The returned object should be spread as props onto the child component
 * @hidden
 */
function useUiCanExitClassComponentHook(router: UIRouter, stateName: string, maybeComponentClass: any) {
  // Use refs and run the callback outside of any render pass
  const componentInstanceRef = ref<any>();
  const deregisterRef = ref<Function>(() => undefined);

  function callbackRef(componentInstance: any) {
    // Use refs
    const previous = componentInstanceRef.value;
    const deregisterPreviousTransitionHook = deregisterRef.value;

    if (previous !== componentInstance) {
      componentInstanceRef.value = componentInstance;
      deregisterPreviousTransitionHook();

      const uiCanExit = componentInstance?.uiCanExit;
      if (uiCanExit) {
        const boundCallback = uiCanExit.bind(componentInstance);
        deregisterRef.value = router.transitionService.onBefore({exiting: stateName}, boundCallback);
      } else {
        deregisterRef.value = () => undefined;
      }
    }
  }

  // useMemo(
  const isComponentClass = maybeComponentClass?.prototype?.render || maybeComponentClass?.render;
  return isComponentClass ? {ref: callbackRef} : undefined;
}

interface ActiveUIView {
  $type: String
  id: Number,
  name: String
  fqn: String
  creationContext: any
  configUpdated: any
  config: ViewConfig
}

/**
 * UIView Viewport
 *
 * The UIView component is a viewport for a routed components.
 * Routed components will be rendered inside the UIView viewport.
 *
 * ### Example
 * ```
 * function MyApp() {
 *   return (
 *     <div className="MyApp">
 *       <UIView />
 *     </div>
 *   );
 * }
 * ```
 *
 * See [[UIViewProps]] for details on the props this component takes.
 *
 * @noInheritDoc
 */
export default {
  name: 'UIView',

  // https://dev.to/voluntadpear/comparing-react-hooks-with-vue-composition-api-4b32
  // https://v3.vuejs.org/guide/composition-api-introduction.html#basics-of-composition-api
  setup(props: UIViewProps, context: any) {
    let ChildOrRenderFunction: any;
    watch([props, context], () => {
      const {children, render, className, style} = props;

      // extract router from context
      debugger;
      //const parent = useRouter();
      const router = context.router;

      // /** @internal Gets the parent UIViewAddress from context, or the root UIViewAddress */
      //const parent = useParentView();
      const parent = context.parent;

      const creationContext = parent.context;

      const {viewConfig, configUpdated} = useViewConfig();

      // useMemo(() =>
      const component = viewConfig.value?.viewDecl?.component;

      const name = props.name || '$default';
      const fqn = parent.fqn ? parent.fqn + '.' + name : name;

      // useMemo(() =>
      const id = ++viewIdCounter;

      // This object contains all the metadata for this UIView
      // useMemo(() =>
      const uiViewData: ActiveUIView = {
        $type: 'vue',
        id,
        name,
        fqn,
        creationContext,
        configUpdated,
        config: viewConfig.value as ViewConfig
      };
      const viewContext: ViewContext | undefined = viewConfig.value?.viewDecl?.$context;
      const stateName: string = viewContext ? viewContext.name : '';
      //const uiViewAddress: UIViewAddress = {fqn, context: viewContext};

      // useMemo(() =>
      const resolveContext = new ResolveContext(viewConfig.value ? viewConfig.value.path : []);

      // useMemo(() =>
      const injector = resolveContext?.injector();

      // useMemo(() =>
      const transition = injector?.get(Transition);
      const resolves = useResolvesWithStringTokens(resolveContext, injector);

      const childProps = useRoutedComponentProps(
        router,
        stateName,
        viewConfig as Ref<ViewConfig>,
        component as VueComponent,
        resolves,
        className as string,
        style as object,
        transition
      ) as VNodeProps;

      // Register/deregister any time the uiViewData changes
      watch([uiViewData], () => router.viewService.registerUIView(uiViewData));

      const childElement =
        !component && isVNode(children)
          ? cloneVNode(children, childProps)
          : createVNode(component || 'div', childProps);

      // if a render function is passed, use that. otherwise render the component normally
      ChildOrRenderFunction =
        typeof render !== 'undefined' && component ? render(component, childProps) : childElement;
    }, {immediate:true});

    return () => ChildOrRenderFunction;
  }
}
