import { useParams } from 'react-router-dom';
import type { Component } from 'svelte';
import { SvelteIsland } from '@shared/presentation/ui/svelte/SvelteIsland.tsx';

/** Route parameters, as react-router hands them over: always strings. */
export type RouteParams = Record<string, string | undefined>;

/**
 * Turns a Svelte page into something `ModuleRoute.lazy` can return.
 *
 * ```ts
 * lazy: async () => ({
 *   Component: sveltePage((await import('./presentation/ui/Secret.page.svelte')).default),
 * }),
 * ```
 *
 * A migrated module changes one line — the rest of its `ScyllaModule` is
 * untouched, so `permission`, `breadcrumb` and `nav` keep working and
 * `module-permissions.test.ts` keeps checking them.
 *
 * **Route params arrive as props.** They are the one thing a page cannot read
 * from a singleton: they live in the React router's state, and reading
 * `window.location` instead would mean re-implementing the path matching. So
 * the wrapper calls `useParams()` and the Svelte page declares what it needs:
 *
 * ```svelte
 * let { projectId }: { projectId?: string } = $props();
 * ```
 *
 * Every value is optional, because a param is only present on the routes that
 * declare it — the same contract `useParams()` has.
 *
 * Disappears in Phase 6 with the rest of the bridge, when the router hands
 * params to a Svelte page directly.
 */
export const sveltePage = <TProps extends RouteParams>(component: Component<TProps>) => {
  const SveltePage = () => {
    // `useParams` returns a fresh object per render, so the island re-assigns
    // its props each time. That is an `Object.assign` of two strings, against a
    // component that only re-renders when the route changes — memoizing it
    // would cost more than it saves.
    const params = useParams() as TProps;

    return (
      // The host node belongs to React, so it has to carry the layout the page
      // would have had as a direct child of `AnimatedOutlet`'s `<main>`:
      // without `min-h-0` a page that scrolls its own table grows the shell
      // instead.
      <SvelteIsland
        component={component}
        props={params}
        className='flex h-full w-full min-h-0 flex-col'
      />
    );
  };

  return SveltePage;
};
