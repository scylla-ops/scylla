import { render, screen } from '@testing-library/svelte';
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

/**
 * Finds an element bits-ui rendered into a floating layer — a tooltip's
 * content, a select's options, a dropdown's items.
 *
 * Two jsdom facts make the plain query useless here, and both come from the
 * same place. bits-ui positions floating content with floating-ui, which has no
 * layout to measure in jsdom and therefore leaves the wrapper at
 * `visibility: hidden` forever:
 *
 * 1. `getByRole` skips the subtree as inaccessible — hence `hidden: true`;
 * 2. the accessible-*name* algorithm ignores text inside a hidden subtree, so
 *    `{ name: 'Pro' }` matches nothing however the option is labelled. That is
 *    why `name` here is compared against the element's text rather than passed
 *    through to testing-library.
 *
 * In a browser the wrapper becomes visible as soon as it is positioned and both
 * problems disappear, so this is an artifact of the environment and not
 * something the component gets wrong. Still query by role: falling back to
 * `getByText` would keep passing if the element lost the role its behaviour
 * advertises, which is exactly the regression these ports kept introducing.
 */
/** Whatever `findAllByRole` accepts — derived rather than imported, because
 *  `@testing-library/dom` is a transitive dependency, not one we declare. */
type RoleMatcher = Parameters<typeof screen.findAllByRole>[0];

export const findFloating = async (role: RoleMatcher, name?: string): Promise<HTMLElement> => {
  const candidates = await screen.findAllByRole(role, { hidden: true });
  const matches =
    name === undefined
      ? candidates
      : candidates.filter(element => element.textContent?.trim() === name);

  if (matches.length !== 1) {
    const described = name === undefined ? String(role) : `${String(role)} named "${name}"`;
    throw new Error(`Expected exactly one ${described} in a floating layer, found ${matches.length}`);
  }

  return matches[0];
};

export const findTooltip = (): Promise<HTMLElement> => findFloating('tooltip');

export const queryTooltip = (): HTMLElement | null =>
  screen.queryByRole('tooltip', { hidden: true });
