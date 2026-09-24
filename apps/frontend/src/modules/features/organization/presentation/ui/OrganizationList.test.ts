import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { installTestNavigator } from '@/test/navigator.ts';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { OrganizationRepository } from '../../domain/repository/organization.repository.ts';
import OrganizationList from './OrganizationList.svelte';
import OrganizationListFixture from './OrganizationList.fixture.svelte';

vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const organizations = [
  { id: 'org-1', name: 'Acme Corp', description: 'the first' },
  { id: 'org-2', name: 'Globex Inc' },
];

let teardown: Array<() => void> = [];
let navigator: ReturnType<typeof installTestNavigator>;
let remove: ReturnType<typeof vi.fn>;

const grant = (...permissions: Permission[]) =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

const setUp = (getMine: () => Promise<ScyllaResult<typeof organizations>>) => {
  remove = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const repository = { getMine, delete: remove } as unknown as OrganizationRepository;
  const cache = withQueryClient();
  teardown = [cache.restore, withRegistry({ organization: { organizationRepository: repository } })];
};

beforeEach(() => {
  vi.clearAllMocks();
  navigator = installTestNavigator();
  permissionsStore.setState({ permissions: null });
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
});

afterEach(() => {
  teardown.forEach(restore => restore());
  navigator.restore();
});

describe('OrganizationList', () => {
  it('shows no organization while the list is loading', () => {
    setUp(() => new Promise(() => {}));
    render(OrganizationList);

    expect(screen.queryByText('Acme Corp')).not.toBeInTheDocument();
  });

  it('lists the organizations of the user', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    expect(await screen.findByText('Acme Corp')).toBeInTheDocument();
    expect(screen.getByText('the first')).toBeInTheDocument();
    expect(screen.getByText('Globex Inc')).toBeInTheDocument();
  });

  it('makes a selected organization active and opens its dashboard', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    await fireEvent.click(await screen.findByText('Globex Inc'));

    expect(contextStore.getState().organization).toEqual({ id: 'org-2', name: 'Globex Inc' });
    expect(navigator.navigate).toHaveBeenCalledWith('/globex-inc/dashboard', undefined);
  });

  it('hides the row actions that the user may not use', async () => {
    grant();
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);
    await screen.findByText('Acme Corp');

    expect(screen.queryByRole('button', { name: 'Members' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Edit' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
  });

  it('opens the members of an organization without also selecting the row', async () => {
    grant(Permission.LIST_ORGANIZATION_MEMBERS);
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    const [, globexMembers] = await screen.findAllByRole('button', { name: 'Members' });
    await userEvent.click(globexMembers);

    expect(navigator.navigate).toHaveBeenCalledTimes(1);
    expect(navigator.navigate).toHaveBeenCalledWith('/globex-inc/members', undefined);
    expect(contextStore.getState().organization.id).toBe('org-2');
  });

  it('opens the edit dialog for an organization', async () => {
    grant(Permission.UPDATE_ORGANIZATION);
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    const [acmeEdit] = await screen.findAllByRole('button', { name: 'Edit' });
    await userEvent.click(acmeEdit);

    expect(await screen.findByRole('dialog', { name: 'Edit organization' })).toBeInTheDocument();
  });

  it('moves to another organization after deleting the active one', async () => {
    grant(Permission.DELETE_ORGANIZATION);
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    const [acmeDelete] = await screen.findAllByRole('button', { name: 'Delete' });
    await userEvent.click(acmeDelete);
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(remove).toHaveBeenCalledWith('org-1'));
    await waitFor(() =>
      expect(contextStore.getState().organization).toEqual({ id: 'org-2', name: 'Globex Inc' }),
    );
    expect(navigator.navigate).toHaveBeenCalledWith('/globex-inc/dashboard', undefined);
  });

  it('keeps the active organization after deleting another one', async () => {
    grant(Permission.DELETE_ORGANIZATION);
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationList);

    const [, globexDelete] = await screen.findAllByRole('button', { name: 'Delete' });
    await userEvent.click(globexDelete);
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));

    await waitFor(() => expect(remove).toHaveBeenCalledWith('org-2'));
    expect(contextStore.getState().organization.id).toBe('org-1');
    expect(navigator.navigate).not.toHaveBeenCalled();
  });

  it('renders each row with the row component that the caller gives', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success(organizations)));
    render(OrganizationListFixture);

    await screen.findByText('Acme Corp');
    expect(screen.getAllByTestId('custom-row')).toHaveLength(2);

    await userEvent.click(screen.getAllByRole('button', { name: 'pick' })[1]);
    expect(navigator.navigate).toHaveBeenCalledWith('/globex-inc/dashboard', undefined);
  });
});
