import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import { PermissionScope } from '@platform/authz';
import {
  MemberRoleOrigin,
  type MemberRole,
} from '../../../../domain/structs/scope-member.struct.ts';
import MemberCard from './MemberCard.svelte';

const role = (overrides: Partial<MemberRole> = {}): MemberRole => ({
  grantId: 'grant-1',
  roleId: 'role-1',
  origin: MemberRoleOrigin.DIRECT,
  scope: PermissionScope.PROJECT,
  ...overrides,
});

const baseProps = {
  name: 'ravenne',
  isCurrentUser: false,
  canRemove: true,
  addableRoles: [],
  onAddRole: vi.fn(),
  labelFor: (roleId: string) => roleId,
  canManage: false,
  disabled: false,
  onRevokeRole: vi.fn(),
  removeTooltip: 'Remove',
  onRemove: vi.fn(),
  roles: [] as MemberRole[],
};

describe('MemberCard', () => {
  it('shows "No role" when the member holds none', () => {
    render(MemberCard, { ...baseProps });
    expect(screen.getAllByText('No role')).toHaveLength(2);
  });

  it('pluralizes the role count in the subtitle', async () => {
    const { rerender } = render(MemberCard, { ...baseProps, roles: [role()] });
    expect(screen.getByText('1 role')).toBeInTheDocument();

    await rerender({ roles: [role({ grantId: 'g1' }), role({ grantId: 'g2' })] });
    expect(screen.getByText('2 roles')).toBeInTheDocument();
  });

  it("marks the current user's own card instead of offering to remove them", () => {
    render(MemberCard, { ...baseProps, isCurrentUser: true });

    expect(screen.getByText('You')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('clicking the remove action calls onRemove', async () => {
    const onRemove = vi.fn();
    render(MemberCard, { ...baseProps, onRemove });

    await userEvent.click(screen.getByRole('button'));

    expect(onRemove).toHaveBeenCalled();
  });

  it('hides the "add a role" footer entirely when the caller cannot manage this scope', () => {
    render(MemberCard, {
      ...baseProps,
      canManage: false,
      addableRoles: [{ roleId: 'r1', name: 'Developer', description: '' }],
    });

    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('offers addable roles in the footer when the caller can manage, and forwards the pick', async () => {
    const onAddRole = vi.fn();
    render(MemberCard, {
      ...baseProps,
      canManage: true,
      addableRoles: [{ roleId: 'r1', name: 'Developer', description: '' }],
      onAddRole,
    });

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'Developer'));

    expect(onAddRole).toHaveBeenCalledWith('r1');
  });
});
