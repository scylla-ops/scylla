import { render } from '@testing-library/svelte';
import { createRawSnippet, type Snippet } from 'svelte';
import { setDependencyRegistry, type DomainRegistry } from '@platform/di';

/**
 * Svelte counterpart of `render.tsx`, and deliberately much smaller.
 *
 * A React component needs four providers around it before it can do anything —
 * i18n, query client, DI. A Svelte component needs none: Phase 0 turned all of
 * them into module singletons, so `render()` from `@testing-library/svelte`
 * works on its own and the helpers below only exist for the pieces that *are*
 * still per-test state.
 *
 * `setup.ts` is shared with the React suite and already activates an empty `en`
 * catalog, which makes lingui fall back to the message id — the English source
 * string. Assertions therefore read `getByText('Create grant')` and stay
 * legible, exactly as on the React side.
 */

export { render };

/**
 * Installs a stub registry for a test that reaches a repository.
 *
 * Module state, not context: it leaks between tests unless cleared, which is
 * what the returned function is for.
 *
 * ```ts
 * const restore = withRegistry({ secret: { secretRepository } });
 * afterEach(restore);
 * ```
 */
export const withRegistry = (registry: DomainRegistry): (() => void) => {
  setDependencyRegistry(registry);
  return () => setDependencyRegistry(null);
};

/**
 * Text as a `children` snippet.
 *
 * `children` is a snippet in Svelte 5, not a node, so a test cannot simply pass
 * a string — every component test with content needs this. The span is there to
 * give the snippet a single root, which `createRawSnippet` requires.
 */
export const textSnippet = (text: string): Snippet =>
  createRawSnippet(() => ({ render: () => `<span>${text}</span>` }));
