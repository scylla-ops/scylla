import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { contextStore } from '@platform/context';
import { installTestNavigator } from '@/test/navigator.ts';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { WHATS_NEW } from '@/modules/layout/whats-new.ts';
import LayoutFixture from './Layout.fixture.svelte';

const syncMyPermissions = vi.fn();

vi.mock('@/modules/features/roles', () => ({
  syncMyPermissions: (organizationId: string | null, projectId: string | null) =>
    syncMyPermissions(organizationId, projectId),
}));
vi.mock('./AppSidebar.svelte', async () => ({
  default: (await import('./Stub.fixture.svelte')).default,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

let teardown: Array<() => void> = [];
let navigator: ReturnType<typeof installTestNavigator>;
let create: ReturnType<typeof vi.fn>;

const setUp = (getMine: () => Promise<ScyllaResult<{ id: string; name: string }[]>>) => {
  create = vi.fn((name: string) => Promise.resolve(ScyllaResult.success({ id: 'org-9', name })));
  const cache = withQueryClient();
  teardown = [
    cache.restore,
    withRegistry({ organization: { organizationRepository: { getMine, create } } }),
  ];
  return render(LayoutFixture);
};

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  localStorage.setItem(`whats-new-seen:${WHATS_NEW.version}:release`, '1');
  navigator = installTestNavigator();
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: 'project-1', name: 'web' },
  });
});

afterEach(() => {
  teardown.forEach(restore => restore());
  navigator.restore();
});

describe('Layout', () => {
  it('shows neither the page nor the welcome screen while the organizations load', () => {
    setUp(() => new Promise(() => {}));

    expect(screen.queryByText('page content')).not.toBeInTheDocument();
    expect(screen.queryByText('Welcome to Scylla!')).not.toBeInTheDocument();
  });

  it('renders the page inside the shell when the user has organizations', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }])));

    expect(await screen.findByText('page content')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle Sidebar' })).toBeInTheDocument();
  });

  it('loads the permissions for the active organization and project', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }])));
    await screen.findByText('page content');

    expect(syncMyPermissions).toHaveBeenCalledWith('org-1', 'project-1');
  });

  it('loads the permissions again when the active project changes', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }])));
    await screen.findByText('page content');

    contextStore.getState().setProject('project-2', 'api');

    await waitFor(() => expect(syncMyPermissions).toHaveBeenCalledWith('org-1', 'project-2'));
  });

  it('shows the welcome screen when the user has no organization', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success([])));

    expect(await screen.findByText('Welcome to Scylla!')).toBeInTheDocument();
    expect(screen.queryByText('page content')).not.toBeInTheDocument();
  });

  it('creates the first organization and opens the settings of the user in it', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success([])));
    await screen.findByText('Welcome to Scylla!');
    await focusSettled();
    const user = userEvent.setup();

    await user.type(screen.getByLabelText('Organization name'), 'My Org');
    await user.type(screen.getByLabelText('Description'), 'Our team');
    await user.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() => expect(create).toHaveBeenCalledWith('My Org', 'Our team'));
    await waitFor(() =>
      expect(navigator.navigate).toHaveBeenCalledWith('/my-org/users/me', undefined),
    );
  });

  it('announces the release once, until the user dismisses it', async () => {
    localStorage.clear();
    setUp(() => Promise.resolve(ScyllaResult.success([{ id: 'org-1', name: 'Acme' }])));

    expect(await screen.findByText(`What's new in Scylla ${WHATS_NEW.version}`)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Got it' }));

    await waitFor(() =>
      expect(screen.queryByText(`What's new in Scylla ${WHATS_NEW.version}`)).not.toBeInTheDocument(),
    );
  });
});
