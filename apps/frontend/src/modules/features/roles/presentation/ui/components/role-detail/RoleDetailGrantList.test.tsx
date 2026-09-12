import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { usePermissionsStore, PermissionScope, PrincipalKind } from '@platform/authz';
import { RoleDetailGrantList } from './RoleDetailGrantList';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';

const assigneesState: { assignees: { grant: GrantEntity; label: string }[] } = { assignees: [] };
const removeAssigneeMock = vi.fn();
vi.mock('@/modules/features/roles/presentation/hooks/use-role-assignees.ts', () => ({
  useRoleAssignees: () => ({ assignees: assigneesState.assignees, removeAssignee: removeAssigneeMock }),
}));

vi.mock('@/modules/features/roles/presentation/hooks/use-grant-target-labels.ts', () => ({
  useGrantTargetLabels: () => ({ labelFor: (id: string) => ({ name: `target-${id}`, resolved: true }) }),
}));

vi.mock(
  '@/modules/features/roles/presentation/ui/components/role-detail/GrantCreator.tsx',
  () => ({ default: () => <button type='button'>Add grant</button> }),
);

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const role: RoleEntity = {
  id: 'role-1',
  name: 'Developer',
  description: '',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
};

const grant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.PROJECT,
  scopeId: 'project-1',
  ...overrides,
});

beforeEach(() => {
  assigneesState.assignees = [];
  removeAssigneeMock.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('RoleDetailGrantList', () => {
  it('says nobody holds the role yet, with the count at zero', () => {
    renderWithI18n(<RoleDetailGrantList role={role} />);
    expect(screen.getByText('Grants (0)')).toBeInTheDocument();
    expect(screen.getByText('No one holds this role yet.')).toBeInTheDocument();
  });

  it('lists each assignee by their resolved label and scope target', () => {
    assigneesState.assignees = [{ grant: grant(), label: 'alice' }];
    renderWithI18n(<RoleDetailGrantList role={role} />);
    expect(screen.getByText('Grants (1)')).toBeInTheDocument();
    expect(screen.getByText('alice')).toBeInTheDocument();
    expect(screen.getByText('target-project-1')).toBeInTheDocument();
  });

  it('revoking a grant calls removeAssignee with its id', async () => {
    assigneesState.assignees = [{ grant: grant({ id: 'grant-42' }), label: 'alice' }];
    const user = userEvent.setup();
    renderWithI18n(<RoleDetailGrantList role={role} />);

    // Two buttons on the page: GrantCreator's "Add grant" (mocked) and this
    // row's revoke IconButton - the revoke one is the last rendered.
    const buttons = screen.getAllByRole('button');
    await user.click(buttons[buttons.length - 1]);
    expect(removeAssigneeMock).toHaveBeenCalledWith('grant-42');
  });

  it('the revoke button is disabled without MANAGE_SYSTEM_GRANTS', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    assigneesState.assignees = [{ grant: grant(), label: 'alice' }];
    renderWithI18n(<RoleDetailGrantList role={role} />);

    const buttons = screen.getAllByRole('button');
    expect(buttons[buttons.length - 1]).toBeDisabled();
  });
});
