import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MemberRoleBadges } from './MemberRoleBadges';
import { MemberRoleOrigin } from '@/modules/features/membership/domain/structs/scope-member.struct.ts';
import type { MemberRole } from '@/modules/features/membership/domain/structs/scope-member.struct.ts';
import { PermissionScope } from '@platform/authz';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const role = (overrides: Partial<MemberRole> = {}): MemberRole => ({
  grantId: 'grant-1',
  roleId: 'role-1',
  origin: MemberRoleOrigin.DIRECT,
  scope: PermissionScope.PROJECT,
  ...overrides,
});

const labelFor = (roleId: string) => `Label for ${roleId}`;

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('MemberRoleBadges', () => {
  it('shows "No role" (default) when the member holds none', () => {
    renderWithI18n(<MemberRoleBadges roles={[]} labelFor={labelFor} />);
    expect(screen.getByText('No role')).toBeInTheDocument();
  });

  it('shows a custom empty message instead', () => {
    renderWithI18n(<MemberRoleBadges roles={[]} labelFor={labelFor} empty='Not yet assigned' />);
    expect(screen.getByText('Not yet assigned')).toBeInTheDocument();
  });

  it('renders one chip per role, labeled through labelFor', () => {
    renderWithI18n(
      <MemberRoleBadges roles={[role({ roleId: 'a' }), role({ roleId: 'b', grantId: 'grant-2' })]} labelFor={labelFor} />,
    );
    expect(screen.getByText('Label for a')).toBeInTheDocument();
    expect(screen.getByText('Label for b')).toBeInTheDocument();
  });

  it('a direct role has no lock icon and, with canManage, a revoke control', () => {
    const { container } = renderWithI18n(
      <MemberRoleBadges roles={[role({ origin: MemberRoleOrigin.DIRECT })]} labelFor={labelFor} canManage />,
    );
    expect(container.querySelector('.lucide-lock')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: /revoke/i })).toBeInTheDocument();
  });

  it('a direct role has no revoke control when canManage is false', () => {
    renderWithI18n(<MemberRoleBadges roles={[role({ origin: MemberRoleOrigin.DIRECT })]} labelFor={labelFor} />);
    expect(screen.queryByRole('button', { name: /revoke/i })).not.toBeInTheDocument();
  });

  it('calls onRevoke with the role when its revoke button is clicked', async () => {
    const onRevoke = vi.fn();
    const user = userEvent.setup();
    const theRole = role({ roleId: 'admin' });
    renderWithI18n(<MemberRoleBadges roles={[theRole]} labelFor={labelFor} canManage onRevoke={onRevoke} />);

    await user.click(screen.getByRole('button', { name: /revoke/i }));

    expect(onRevoke).toHaveBeenCalledWith(theRole);
  });

  it('disables the revoke control when asked, without calling onRevoke', async () => {
    const onRevoke = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <MemberRoleBadges roles={[role()]} labelFor={labelFor} canManage disabled onRevoke={onRevoke} />,
    );

    const button = screen.getByRole('button', { name: /revoke/i });
    expect(button).toBeDisabled();
    await user.click(button);
    expect(onRevoke).not.toHaveBeenCalled();
  });

  it('an inherited role is locked (no revoke, even with canManage) and shows its scope', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <MemberRoleBadges
        roles={[role({ origin: MemberRoleOrigin.INHERITED, scope: PermissionScope.ORGANIZATION })]}
        labelFor={labelFor}
        canManage
      />,
    );

    expect(screen.queryByRole('button', { name: /revoke/i })).not.toBeInTheDocument();
    expect(screen.getByText('Organization')).toBeInTheDocument();

    await user.hover(screen.getByText('Label for role-1'));
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('Managed at the organization level.'));
  });
});
