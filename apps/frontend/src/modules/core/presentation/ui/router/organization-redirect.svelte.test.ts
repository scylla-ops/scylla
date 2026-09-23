import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { useContextStore } from '@platform/context';
import { stubQuery } from '@/test/queries.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { withQueryClient } from '@/test/render.svelte.ts';
import { redirectToOrganization } from './organization-redirect.svelte.ts';

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

const run = () => {
  const cleanup = $effect.root(() => {
    redirectToOrganization();
  });
  flushSync();
  cleanup();
};

beforeEach(() => {
  navigator = installTestNavigator();
  cache = withQueryClient();
  organizationsState.organizations = undefined;
  organizationsState.isLoading = false;
  useContextStore.setState({ organization: { id: null, name: null } });
});

afterEach(() => {
  navigator.restore();
  cache.restore();
});

describe('redirectToOrganization', () => {
  it('waits while the organizations are loading', () => {
    organizationsState.isLoading = true;
    run();

    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('goes to the dashboard of the active organization', () => {
    useContextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
    organizationsState.organizations = [{ id: 'org-2', name: 'Other Co' }];
    run();

    expect(navigator.navigate).toHaveBeenCalledWith('/acme-corp/dashboard', { replace: true });
  });

  it('goes to the first organization when none is active', () => {
    organizationsState.organizations = [
      { id: 'org-1', name: 'Globex Inc' },
      { id: 'org-2', name: 'Other Co' },
    ];
    run();

    expect(navigator.navigate).toHaveBeenCalledWith('/globex-inc/dashboard', { replace: true });
  });

  it('stays when there is no organization at all', () => {
    organizationsState.organizations = [];
    run();

    expect(navigator.navigate).not.toHaveBeenCalled();
  });
});
