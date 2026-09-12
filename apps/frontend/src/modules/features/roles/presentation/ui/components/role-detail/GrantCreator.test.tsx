import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { usePermissionsStore, PermissionScope, PrincipalKind } from '@platform/authz';
import { GrantCreator } from './GrantCreator';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';

const grantsState: { grants: GrantEntity[] } = { grants: [] };
const createGrantMock = vi.fn().mockResolvedValue(undefined);
vi.mock('@/modules/features/roles/presentation/hooks/use-grants.ts', () => ({
  useGrants: () => ({
    grants: grantsState.grants,
    createGrant: { mutateAsync: createGrantMock, isPending: false },
  }),
}));

const eligibilityForMock = vi.fn().mockReturnValue('eligible');
vi.mock('@/modules/features/roles/presentation/hooks/use-project-grant-eligibility.ts', () => ({
  useProjectGrantEligibility: () => ({ eligibilityFor: eligibilityForMock }),
}));

const usersState: { users: { userId: string; username: string }[] } = {
  users: [
    { userId: 'user-1', username: 'alice' },
    { userId: 'user-2', username: 'bob' },
  ],
};
vi.mock('@/modules/features/user', () => ({
  useUsers: () => ({ users: { items: usersState.users } }),
}));

const orgsState: { organizations: { id: string; name: string }[] } = {
  organizations: [
    { id: 'org-1', name: 'Acme' },
    { id: 'org-2', name: 'Globex' },
  ],
};
vi.mock('@/modules/features/organization', () => ({
  useOrganizations: () => ({ organizations: orgsState.organizations, isLoading: false }),
}));

const projectsState: { projects: { id: string; name: string }[] } = {
  projects: [{ id: 'project-1', name: 'web' }],
};
vi.mock('@/modules/features/project', () => ({
  useProjects: () => ({ projects: projectsState.projects, isLoading: false }),
}));

const toastSuccess = vi.fn();
const toastError = vi.fn();
vi.mock('@shared/presentation/utils/toast.ts', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args), error: (...args: unknown[]) => toastError(...args) },
}));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Developer',
  description: '',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

beforeEach(() => {
  grantsState.grants = [];
  usersState.users = [
    { userId: 'user-1', username: 'alice' },
    { userId: 'user-2', username: 'bob' },
  ];
  orgsState.organizations = [
    { id: 'org-1', name: 'Acme' },
    { id: 'org-2', name: 'Globex' },
  ];
  projectsState.projects = [{ id: 'project-1', name: 'web' }];
  createGrantMock.mockClear().mockResolvedValue(undefined);
  eligibilityForMock.mockReset().mockReturnValue('eligible');
  toastSuccess.mockClear();
  toastError.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

const openDialog = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(screen.getByRole('button', { name: /add grant/i }));
};

const selectUser = async (user: ReturnType<typeof userEvent.setup>, name: string) => {
  await user.click(screen.getByRole('combobox', { name: /user/i }));
  await user.click(await screen.findByText(name));
};

describe('GrantCreator', () => {
  it('opens the dialog with the role name in the title', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ name: 'Developer' })} />);
    await openDialog(user);
    expect(screen.getByText(/Grant.*Developer/)).toBeInTheDocument();
  });

  it('a SYSTEM-scoped role shows only the user picker plus a system-wide notice, no target checklist', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.SYSTEM })} />);
    await openDialog(user);

    expect(screen.getByText('This role grants access across the whole system.')).toBeInTheDocument();
    expect(screen.queryByText('Organizations')).not.toBeInTheDocument();
  });

  it('a SYSTEM-scoped grant is created with an empty scopeId', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ id: 'role-sys', scope: PermissionScope.SYSTEM })} />);
    await openDialog(user);
    await selectUser(user, 'alice');
    await user.click(screen.getByRole('button', { name: 'Create grant' }));

    expect(createGrantMock).toHaveBeenCalledWith({
      principal: { kind: PrincipalKind.USER, id: 'user-1' },
      roleId: 'role-sys',
      scope: PermissionScope.SYSTEM,
      scopeId: '',
    });
    expect(toastSuccess).toHaveBeenCalledWith('Grant created');
  });

  it('an ORGANIZATION-scoped role lists organizations to toggle, with a removable "Selected" summary', async () => {
    const user = userEvent.setup();
    const { container } = renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.ORGANIZATION })} />);
    await openDialog(user);

    expect(screen.getByText('Organizations')).toBeInTheDocument();
    await user.click(screen.getByText('Acme'));
    expect(screen.getByText('Selected (1)')).toBeInTheDocument();

    // The Badge's own remove button un-selects the same target - "Acme" also
    // appears in the checklist option above it, so scope the query to the badge.
    await user.click(container.querySelector('[data-slot="badge"] button')!);
    expect(screen.queryByText(/Selected/)).not.toBeInTheDocument();
  });

  it('creates one grant per selected organization and reports the count', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ id: 'role-org', scope: PermissionScope.ORGANIZATION })} />);
    await openDialog(user);
    await selectUser(user, 'alice');
    await user.click(screen.getByText('Acme'));
    await user.click(screen.getByText('Globex'));
    await user.click(screen.getByRole('button', { name: 'Create grant' }));

    expect(createGrantMock).toHaveBeenCalledTimes(2);
    expect(createGrantMock).toHaveBeenCalledWith(
      expect.objectContaining({ scope: PermissionScope.ORGANIZATION, scopeId: 'org-1' }),
    );
    expect(createGrantMock).toHaveBeenCalledWith(
      expect.objectContaining({ scope: PermissionScope.ORGANIZATION, scopeId: 'org-2' }),
    );
    expect(toastSuccess).toHaveBeenCalledWith('2 grants created');
  });

  it('an organization the user already holds this role in is offered pre-checked and disabled, labelled "Granted"', async () => {
    grantsState.grants = [
      {
        id: 'g1',
        principal: { kind: PrincipalKind.USER, id: 'user-1' },
        roleId: 'role-1',
        scope: PermissionScope.ORGANIZATION,
        scopeId: 'org-1',
      },
    ];
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ id: 'role-1', scope: PermissionScope.ORGANIZATION })} />);
    await openDialog(user);
    await selectUser(user, 'alice');

    expect(screen.getByText('Granted')).toBeInTheDocument();
    const checkboxes = screen.getAllByRole('checkbox');
    expect(checkboxes[0]).toBeChecked();
    expect(checkboxes[0]).toBeDisabled();
  });

  it('a PROJECT-scoped role disables the user picker until an organization is picked', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.PROJECT })} />);
    await openDialog(user);

    expect(screen.getByRole('combobox', { name: /^user$/i })).toBeDisabled();
    await user.click(screen.getByRole('combobox', { name: /organization/i }));
    await user.click(await screen.findByText('Acme'));
    expect(screen.getByRole('combobox', { name: /^user$/i })).toBeEnabled();
  });

  it('a project-ineligible user is shown greyed out with the reason, and an org with nobody eligible shows a notice instead of the project list', async () => {
    eligibilityForMock.mockImplementation((userId: string) =>
      userId === 'user-1' ? 'not-admitted' : 'cannot-see-projects',
    );
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.PROJECT })} />);
    await openDialog(user);
    await user.click(screen.getByRole('combobox', { name: /organization/i }));
    await user.click(await screen.findByText('Acme'));

    await user.click(screen.getByRole('combobox', { name: /^user$/i }));
    expect(screen.getByText('not a member of this organization')).toBeInTheDocument();
    expect(screen.getByText("can't see this organization's projects")).toBeInTheDocument();
    expect(
      screen.getByText(/Nobody can receive a project grant here yet/),
    ).toBeInTheDocument();
  });

  it('switching organization drops a user who is no longer eligible in the new one', async () => {
    eligibilityForMock.mockReturnValue('eligible');
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.PROJECT })} />);
    await openDialog(user);
    await user.click(screen.getByRole('combobox', { name: /organization/i }));
    await user.click(await screen.findByText('Acme'));
    await selectUser(user, 'alice');

    eligibilityForMock.mockReturnValue('not-admitted');
    await user.click(screen.getByRole('combobox', { name: /organization/i }));
    await user.click(await screen.findByText('Globex'));

    expect(screen.getByRole('combobox', { name: /^user$/i })).toHaveTextContent('Select a user');
  });

  it('an error from the mutation is toasted and the dialog stays open', async () => {
    createGrantMock.mockRejectedValueOnce(new Error('backend rejected it'));
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role({ scope: PermissionScope.SYSTEM })} />);
    await openDialog(user);
    await selectUser(user, 'alice');
    await user.click(screen.getByRole('button', { name: 'Create grant' }));

    await waitFor(() => expect(toastError).toHaveBeenCalledWith('backend rejected it'));
    expect(screen.getByRole('button', { name: 'Create grant' })).toBeInTheDocument();
  });

  it('Cancel closes the dialog', async () => {
    const user = userEvent.setup();
    renderWithI18n(<GrantCreator role={role()} />);
    await openDialog(user);
    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('button', { name: 'Create grant' })).not.toBeInTheDocument();
  });
});
