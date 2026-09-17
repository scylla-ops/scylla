import { useEffect, useRef } from 'react';
import { mount, unmount, type Component } from 'svelte';

interface SvelteIslandProps<TProps extends Record<string, unknown>> {
  /** The Svelte component, imported as `import X from './X.svelte'`. */
  component: Component<TProps>;
  props: TProps;
}

/**
 * Mounts a Svelte component inside the React tree.
 *
 * The whole bridge, and deliberately this small: nothing is passed down through
 * it. A Svelte island reads the query cache, the DI registry, the theme, the
 * permissions and the locale from module-level singletons, so there is no React
 * context for it to inherit and none to forward — that is what Phase 0 bought,
 * and why this file is 30 lines instead of one adapter per capability.
 *
 * React owns the container element and Svelte owns everything inside it; the two
 * never reconcile the same node. Props are pushed on change rather than the
 * component being remounted, so island state survives a parent re-render.
 *
 * Temporary: an island disappears when its parent is migrated too, and this file
 * goes with the last one — see `refacto_svelte.md` §3.
 */
export const SvelteIsland = <TProps extends Record<string, unknown>>({
  component,
  props,
}: SvelteIslandProps<TProps>) => {
  const host = useRef<HTMLDivElement>(null);
  const instance = useRef<Record<string, unknown> | null>(null);
  const live = useRef<TProps>(props);

  live.current = props;

  // Mount is a subscription to something outside React: Svelte takes the node
  // over and keeps it until told otherwise. `component` is intentionally the
  // only dependency — a new props object on every parent render would otherwise
  // tear the island down and back up, losing its state.
  useEffect(() => {
    if (!host.current) return;

    const mounted = mount(component, { target: host.current, props: live.current });
    instance.current = mounted;

    return () => {
      void unmount(mounted);
      instance.current = null;
    };
  }, [component]);

  // Props are handed over by assignment: `mount` returns the instance exports,
  // and a `$props()` rune reads through to them.
  useEffect(() => {
    if (!instance.current) return;
    Object.assign(instance.current, props);
  }, [props]);

  return <div ref={host} />;
};
