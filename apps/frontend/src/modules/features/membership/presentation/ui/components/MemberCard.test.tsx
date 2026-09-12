import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MemberCard } from './MemberCard';
import { MemberRoleOrigin, type MemberRole } from '@/modules/features/membership/domain/structs/scope-member.struct.ts';
import { PermissionScope } from '@platform/authz';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

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
};

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

describe('MemberCard', () => {
  it('shows "No role" when the member holds none', () => {
    renderWithI18n(<MemberCard {...baseProps} roles={[]} />);
    // Appears twice: the subtitle's pluralized count and MemberRoleBadges' own
    // empty state both say "No role" for a member with zero roles.
    expect(screen.getAllByText('No role')).toHaveLength(2);
  });

  it('pluralizes the role count in the subtitle', () => {
    const { rerender } = renderWithI18n(<MemberCard {...baseProps} roles={[role()]} />);
    expect(screen.getByText('1 role')).toBeInTheDocument();

    rerender(
      <I18nProvider i18n={i18n}>
        <MemberCard {...baseProps} roles={[role({ grantId: 'g1' }), role({ grantId: 'g2' })]} />
      </I18nProvider>,
    );
    expect(screen.getByText('2 roles')).toBeInTheDocument();
  });

  it('marks the current user\'s own card instead of offering to remove them', () => {
    renderWithI18n(<MemberCard {...baseProps} roles={[]} isCurrentUser />);
    expect(screen.getByText('You')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('clicking the remove action calls onRemove', async () => {
    const onRemove = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<MemberCard {...baseProps} roles={[]} onRemove={onRemove} />);
    await user.click(screen.getByRole('button'));
    expect(onRemove).toHaveBeenCalled();
  });

  it('hides the "add a role" footer entirely when the caller cannot manage this scope', () => {
    renderWithI18n(
      <MemberCard
        {...baseProps}
        roles={[]}
        canManage={false}
        addableRoles={[{ roleId: 'r1', name: 'Developer', description: '' }]}
      />,
    );
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('offers addable roles in the footer when the caller can manage, and forwards the pick', async () => {
    const onAddRole = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <MemberCard
        {...baseProps}
        roles={[]}
        canManage
        addableRoles={[{ roleId: 'r1', name: 'Developer', description: '' }]}
        onAddRole={onAddRole}
      />,
    );

    await user.click(screen.getByRole('combobox'));
    await user.click(await screen.findByText('Developer'));
    expect(onAddRole).toHaveBeenCalledWith('r1');
  });
});
