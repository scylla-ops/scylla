import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient as ReactQueryClient } from '@tanstack/react-query';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { setDependencyRegistry } from '@platform/di';
import { queryClient } from '@platform/query';
import { SvelteIsland } from '@shared/presentation/ui/svelte/SvelteIsland.tsx';
import PlatformSingletons from './PlatformSingletons.fixture.svelte';

/**
 * Phase 0's exit criterion, as a test rather than as a throwaway component.
 *
 * A Svelte component mounted inside a React tree sees no React context. These
 * pin that it does not need any: the translation, the permission, the context
 * store and the DI registry all reach it through module-level singletons. Each
 * assertion fails the day one of them goes back behind a provider.
 */
beforeEach(() => {
  setDependencyRegistry({ 'island-fixture': { marker: 'domain reached' } });
  usePermissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.ORGANIZATION,
          scopeId: 'org-1',
          access: { kind: 'restricted', permissions: [Permission.LIST_SECRETS] },
        },
      ],
    },
  });
  useContextStore.getState().setOrganization('org-1', 'Scylla Ops');
});

afterEach(() => {
  setDependencyRegistry(null);
  usePermissionsStore.setState({ permissions: null });
  useContextStore.getState().reset();
});

describe('a Svelte island inside the React tree', () => {
  it('mounts a Svelte component inside the React tree', () => {
    render(<SvelteIsland component={PlatformSingletons} props={{ label: 'from react' }} />);

    expect(screen.getByTestId('greeting')).toHaveTextContent('Island is mounted');
  });

  it('hands props down from React', () => {
    render(<SvelteIsland component={PlatformSingletons} props={{ label: 'from react' }} />);

    expect(screen.getByTestId('label')).toHaveTextContent('from react');
  });

  it('reads the context store without a React provider above it', () => {
    render(<SvelteIsland component={PlatformSingletons} props={{ label: '' }} />);

    expect(screen.getByTestId('organization')).toHaveTextContent('Scylla Ops');
  });

  it('answers a permission check through the same store React reads', () => {
    render(<SvelteIsland component={PlatformSingletons} props={{ label: '' }} />);

    expect(screen.getByTestId('permission')).toHaveTextContent('allowed');
  });

  it('denies when the permissions are still unknown', () => {
    usePermissionsStore.setState({ permissions: null });

    render(<SvelteIsland component={PlatformSingletons} props={{ label: '' }} />);

    expect(screen.getByTestId('permission')).toHaveTextContent('denied');
  });

  it('reaches the DI registry with no DependenciesProvider in the tree', () => {
    render(<SvelteIsland component={PlatformSingletons} props={{ label: '' }} />);

    expect(screen.getByTestId('domain')).toHaveTextContent('domain reached');
  });

  it('tears the island down when React unmounts it', () => {
    const { unmount } = render(<SvelteIsland component={PlatformSingletons} props={{ label: '' }} />);

    unmount();

    expect(screen.queryByTestId('greeting')).toBeNull();
  });
});

describe('the shared query cache', () => {
  /**
   * `react-query` and `svelte-query` pin `query-core` to an exact version each,
   * and two copies fork the cache in silence: nothing throws, no test fails, and
   * a Svelte mutation simply stops invalidating a React query.
   *
   * `queryClient` is built from `query-core` directly, so it is only an instance
   * of what react-query re-exports when both resolve the same copy. That makes
   * this one assertion a detector for the whole hazard.
   */
  it('is the one instance both bindings recognise', () => {
    expect(queryClient).toBeInstanceOf(ReactQueryClient);
  });
});
