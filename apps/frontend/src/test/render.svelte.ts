import { render, screen } from '@testing-library/svelte';
import { createRawSnippet, type Snippet } from 'svelte';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry, type DomainRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';

/** No provider is needed: these helpers install the per-test state. The empty `en` catalog renders the English source. */

export { render };

/** Module state: restore it after the test. */
export const withRegistry = (registry: DomainRegistry): (() => void) => {
  setDependencyRegistry(registry);
  return () => setDependencyRegistry(null);
};

/** A fresh cache with `retry: false`, or a failing query retries until the test times out. Restore it after the test. */
export const withQueryClient = (): { queryClient: QueryClient; restore: () => void } => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  setQueryClient(queryClient);

  return {
    queryClient,
    restore: () => {
      queryClient.clear();
      setQueryClient(null);
    },
  };
};

/**
 * Call it after opening a dialog, before typing: bits-ui moves the focus
 * asynchronously and would cut a `user.type()` in the middle.
 */
export const focusSettled = async (): Promise<void> => {
  // A macrotask, not `waitFor`: its checks can all land inside one task.
  const tick = async () => {
    await new Promise(resolve => requestAnimationFrame(() => resolve(null)));
    await new Promise(resolve => setTimeout(resolve, 0));
  };

  await tick();
  let previous = document.activeElement;
  await tick();

  while (document.activeElement !== previous) {
    previous = document.activeElement;
    await tick();
  }
};

/** A text `children` snippet, with a single root as `createRawSnippet` requires. */
export const textSnippet = (text: string): Snippet =>
  createRawSnippet(() => ({ render: () => `<span>${text}</span>` }));

/**
 * Finds an element in a bits-ui floating layer. jsdom leaves the layer
 * `visibility: hidden`, so the role query needs `hidden: true` and the name is
 * compared against the text. Still query by role.
 */
/** Derived: `@testing-library/dom` is not a direct dependency. */
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
