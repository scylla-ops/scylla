import type { ReactElement, ReactNode } from 'react';
import { render, renderHook } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DependenciesProvider, type DomainRegistry } from '@platform/di';

/**
 * Render helpers for the suite. Every component in this app sits under at least
 * an `I18nProvider` — `<Trans>` renders nothing useful without one — and every
 * hook that talks to a repository needs both a `QueryClientProvider` and the DI
 * registry. Rebuilding that per file was 57 identical copies of the same four
 * lines, so it lives here instead.
 *
 * The locale is activated in `setup.ts` with an empty catalog, which makes
 * lingui fall back to the message id — the English source string. Assertions
 * therefore read `getByText('Create grant')` and stay legible.
 */

/**
 * A component that only needs translations — the common case.
 *
 * The provider is passed as `wrapper` rather than wrapped around `ui`, so the
 * returned `rerender` re-applies it on its own; wrapping by hand means every
 * `rerender` call site has to repeat the provider or silently lose it.
 */
export const renderWithI18n = (ui: ReactElement) =>
  render(ui, { wrapper: ({ children }) => <I18nProvider i18n={i18n}>{children}</I18nProvider> });

/**
 * `retry: false` matters: without it a query that rejects is retried three times
 * with backoff, and the test times out instead of reporting the error.
 */
export const createTestQueryClient = (): QueryClient =>
  new QueryClient({ defaultOptions: { queries: { retry: false } } });

interface ProviderOptions {
  /** Module id -> stub domain, e.g. `{ roles: { permissionRepository } }`. */
  registry?: DomainRegistry;
  /** Pass one in to assert on the cache after a mutation. */
  queryClient?: QueryClient;
}

/**
 * The full provider stack: i18n + TanStack Query + DI.
 *
 * Returns the `queryClient` alongside the render result so a test can inspect
 * or invalidate the cache without having to build one itself.
 */
export const renderWithProviders = (
  ui: ReactElement,
  { registry = {}, queryClient = createTestQueryClient() }: ProviderOptions = {},
) =>
  Object.assign(
    render(ui, { wrapper: ({ children }) => providers({ children, registry, queryClient }) }),
    { queryClient },
  );

/** `renderHook` counterpart of {@link renderWithProviders}. */
export const renderHookWithProviders = <TProps, TResult>(
  hook: (props: TProps) => TResult,
  { registry = {}, queryClient = createTestQueryClient() }: ProviderOptions = {},
) =>
  Object.assign(
    renderHook(hook, {
      wrapper: ({ children }) => providers({ children, registry, queryClient }),
    }),
    { queryClient },
  );

/**
 * The same stack as a `wrapper` component, for the hook tests that build their
 * own fake repository and want to keep driving `renderHook` themselves.
 *
 * The `queryClient` comes back with it because most of those tests assert on
 * the cache — an invalidation after a mutation, typically.
 */
export const createProvidersWrapper = (registry: DomainRegistry) => {
  const queryClient = createTestQueryClient();
  const Wrapper = ({ children }: { children: ReactNode }) =>
    providers({ children, registry, queryClient });
  return { Wrapper, queryClient };
};

const providers = ({
  children,
  registry,
  queryClient,
}: {
  children: ReactNode;
  registry: DomainRegistry;
  queryClient: QueryClient;
}) => (
  <I18nProvider i18n={i18n}>
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={registry}>{children}</DependenciesProvider>
    </QueryClientProvider>
  </I18nProvider>
);
