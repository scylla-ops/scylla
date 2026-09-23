import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { useContextStore } from '@platform/context';
import { stubQuery } from '@/test/queries.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { withQueryClient } from '@/test/render.svelte.ts';
import { syncOrganization } from './organization-sync.svelte.ts';

const organizationsState: { organizations?: { id: string; name: string }[]; isLoading: boolean } = {
  organizations: undefined,
  isLoading: false,
};

vi.mock('@/modules/features/organization', () => ({
  organizationQueries: {
    mine: () =>
      stubQuery(['organizations', 'mine'], organizationsState.organizations, {
        loading: organizationsState.isLoading,
      }),
  },
}));

let navigator: ReturnType<typeof installTestNavigator>;
let cache: ReturnType<typeof withQueryClient>;

const run = (slug: string | undefined) => {
  const cleanup = $effect.root(() => {
    syncOrganization(slug);
  });
  flushSync();
  return cleanup;
};

beforeEach(() => {
  navigator = installTestNavigator();
  cache = withQueryClient();
  organizationsState.organizations = [
    { id: 'org-1', name: 'Acme Corp' },
    { id: 'org-2', name: 'Globex Inc' },
  ];
  organizationsState.isLoading = false;
  useContextStore.setState({ organization: { id: null, name: null } });
});

afterEach(() => {
  navigator.restore();
  cache.restore();
});

describe('syncOrganization', () => {
  it('does nothing without a slug', () => {
    run(undefined)();

    expect(useContextStore.getState().organization.id).toBeNull();
    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('does nothing while the organizations are still loading', () => {
    organizationsState.isLoading = true;
    run('acme-corp')();

    expect(useContextStore.getState().organization.id).toBeNull();
    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('makes the organization that matches the slug the active one', () => {
    run('globex-inc')();

    expect(useContextStore.getState().organization).toEqual({ id: 'org-2', name: 'Globex Inc' });
    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('does not set the store again when the matched organization is already active', () => {
    useContextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
    const setOrganization = vi.spyOn(useContextStore.getState(), 'setOrganization');

    run('acme-corp')();

    expect(setOrganization).not.toHaveBeenCalled();
  });

  it('goes to the first organization when the slug matches none', () => {
    run('no-such-org')();

    expect(navigator.navigate).toHaveBeenCalledWith('/acme-corp/dashboard', { replace: true });
    expect(useContextStore.getState().organization).toEqual({ id: 'org-1', name: 'Acme Corp' });
  });

  it('does nothing when the slug matches none and there is no organization', () => {
    organizationsState.organizations = [];
    run('no-such-org')();

    expect(navigator.navigate).not.toHaveBeenCalled();
    expect(useContextStore.getState().organization.id).toBeNull();
  });
});
