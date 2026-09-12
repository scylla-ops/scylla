import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RoleFormDialog } from './RoleFormDialog';
import { Permission, PermissionScope } from '@platform/authz';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';

const createRoleState = { mutate: vi.fn(), isPending: false, isSuccess: false };
const updateRoleState = { mutate: vi.fn(), isPending: false, isSuccess: false, reset: vi.fn() };
vi.mock('@/modules/features/roles/presentation/hooks/use-roles.ts', () => ({
  useRoles: () => ({ createRole: createRoleState, updateRole: updateRoleState }),
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
  description: 'writes code',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'restricted', permissions: [Permission.READ_PROJECT] },
  ...overrides,
});

beforeEach(() => {
  createRoleState.mutate.mockReset();
  createRoleState.isPending = false;
  createRoleState.isSuccess = false;
  updateRoleState.mutate.mockReset();
  updateRoleState.isPending = false;
  updateRoleState.isSuccess = false;
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

const selectAccessKind = async (user: ReturnType<typeof userEvent.setup>, label: string) => {
  await user.click(screen.getByLabelText('Access'));
  await user.click(await screen.findByText(label));
};

const selectScope = async (user: ReturnType<typeof userEvent.setup>, label: string) => {
  await user.click(screen.getByLabelText('Scope'));
  await user.click(await screen.findByText(label));
};

describe('RoleFormDialog', () => {
  it('create mode: titles "Create role", defaults to an editable Organization scope', () => {
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);
    // "Create role" appears twice: the dialog title and the submit button.
    expect(screen.getAllByText('Create role')).toHaveLength(2);
    expect(screen.getByLabelText('Scope')).toBeEnabled();
    expect(screen.queryByText('Scope cannot be changed after creation.')).not.toBeInTheDocument();
  });

  it('edit mode: titles "Edit role", prefills the fields and locks the scope', () => {
    renderWithI18n(<RoleFormDialog open role={role()} onClose={vi.fn()} />);
    expect(screen.getByText('Edit role')).toBeInTheDocument();
    expect(screen.getByLabelText('Name')).toHaveValue('Developer');
    expect(screen.getByLabelText('Description')).toHaveValue('writes code');
    expect(screen.getByLabelText('Scope')).toBeDisabled();
    expect(screen.getByText('Scope cannot be changed after creation.')).toBeInTheDocument();
  });

  it('Create is disabled until a name is entered (default scope/access already confers something)', async () => {
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);
    const button = screen.getByRole('button', { name: 'Create role' });
    expect(button).toBeDisabled();

    await user.type(screen.getByLabelText('Name'), 'viewer');
    expect(button).toBeEnabled();
  });

  it('switching access to Full control hides the permission editor entirely', async () => {
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);

    expect(screen.getByText('Permissions')).toBeInTheDocument();
    await selectAccessKind(user, 'Full control');
    expect(screen.queryByText('Permissions')).not.toBeInTheDocument();
  });

  it('a restricted role at PROJECT scope with nothing selected cannot be created (no implicit permission there)', async () => {
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);

    await user.type(screen.getByLabelText('Name'), 'viewer');
    await selectScope(user, 'Project');
    expect(screen.getByRole('button', { name: 'Create role' })).toBeDisabled();
  });

  it('Full control at PROJECT scope needs no permissions ticked to be valid', async () => {
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);

    await user.type(screen.getByLabelText('Name'), 'viewer');
    await selectScope(user, 'Project');
    await selectAccessKind(user, 'Full control');
    expect(screen.getByRole('button', { name: 'Create role' })).toBeEnabled();
  });

  it('creating a role submits the implicit permission and closes on success', async () => {
    createRoleState.mutate.mockImplementation((_vars, opts) => opts.onSuccess());
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={onClose} />);

    await user.type(screen.getByLabelText('Name'), '  viewer  ');
    await user.type(screen.getByLabelText('Description'), '  can view  ');
    await user.click(screen.getByRole('button', { name: 'Create role' }));

    expect(createRoleState.mutate).toHaveBeenCalledWith(
      {
        name: 'viewer',
        description: 'can view',
        scope: PermissionScope.ORGANIZATION,
        access: { kind: 'restricted', permissions: [Permission.READ_ORGANIZATION] },
      },
      expect.any(Object),
    );
    expect(onClose).toHaveBeenCalled();
  });

  it('editing a role updates it by id, with no scope in the payload at all', async () => {
    updateRoleState.mutate.mockImplementation((_vars, opts) => opts.onSuccess());
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={role({ id: 'role-9' })} onClose={onClose} />);

    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    expect(updateRoleState.mutate).toHaveBeenCalledWith(
      {
        id: 'role-9',
        name: 'Developer',
        description: 'writes code',
        access: { kind: 'restricted', permissions: [Permission.READ_PROJECT] },
      },
      expect.any(Object),
    );
    expect(onClose).toHaveBeenCalled();
  });

  it('Cancel closes without submitting', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RoleFormDialog open role={null} onClose={onClose} />);

    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onClose).toHaveBeenCalled();
    expect(createRoleState.mutate).not.toHaveBeenCalled();
  });

  it('disables Cancel and Create while the mutation is pending', () => {
    createRoleState.isPending = true;
    renderWithI18n(<RoleFormDialog open role={null} onClose={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Create role' })).toBeDisabled();
  });
});
