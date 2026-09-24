import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { PermissionScope } from '@platform/authz';
import { findTooltip, render } from '@/test/render.svelte.ts';
import { MemberRoleOrigin, type MemberRole } from '../../../../domain/structs/scope-member.struct.ts';
import MemberRoleBadges from './MemberRoleBadges.svelte';

const role = (overrides: Partial<MemberRole> = {}): MemberRole =>
  ({
    grantId: 'grant-1',
    roleId: 'project-admin',
    origin: MemberRoleOrigin.DIRECT,
    scope: PermissionScope.PROJECT,
    ...overrides,
  });

const props = (overrides: Record<string, unknown> = {}) => ({
  roles: [role()],
  labelFor: (roleId: string) => roleId,
  canManage: false,
  onRevoke: vi.fn(),
  ...overrides,
});

describe('MemberRoleBadges', () => {
  it('says the member holds no role when the list is empty', () => {
    render(MemberRoleBadges, props({ roles: [] }));
    expect(screen.getByText('No role')).toBeInTheDocument();
  });

  it('prefers a caller-supplied empty message', () => {
    render(MemberRoleBadges, props({ roles: [], empty: 'Owner, with no explicit role' }));
    expect(screen.getByText('Owner, with no explicit role')).toBeInTheDocument();
  });

  it('labels each chip through labelFor rather than showing the raw id', () => {
    render(MemberRoleBadges, props({ labelFor: () => 'Project admin' }));
    expect(screen.getByText('Project admin')).toBeInTheDocument();
  });

  it('offers no revoke control when the caller may not manage the scope', () => {
    render(MemberRoleBadges, props({ canManage: false }));
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('names the revoke control after the role it would remove', () => {
    render(MemberRoleBadges, props({ canManage: true, labelFor: () => 'Project admin' }));

    expect(screen.getByRole('button', { name: 'Revoke Project admin' })).toBeInTheDocument();
  });

  it('reports the whole role, not just its id, when revoked', async () => {
    const onRevoke = vi.fn();
    const target = role({ grantId: 'grant-9' });
    render(MemberRoleBadges, props({ roles: [target], canManage: true, onRevoke }));

    await userEvent.click(screen.getByRole('button'));

    expect(onRevoke).toHaveBeenCalledWith(expect.objectContaining({ grantId: 'grant-9' }));
  });

  it('disables revoking while a write is in flight', async () => {
    const onRevoke = vi.fn();
    render(MemberRoleBadges, props({ canManage: true, disabled: true, onRevoke }));

    const button = screen.getByRole('button');
    expect(button).toBeDisabled();

    await userEvent.click(button);
    expect(onRevoke).not.toHaveBeenCalled();
  });

  it('shows an inherited role locked, never revocable here, even with canManage', () => {
    render(
      MemberRoleBadges,
      props({
        roles: [role({ origin: MemberRoleOrigin.INHERITED, scope: PermissionScope.ORGANIZATION })],
        canManage: true,
      }),
    );

    // Hidden would make the project look like it grants less access than it does.
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('explains where an inherited role is administered', async () => {
    render(
      MemberRoleBadges,
      props({
        roles: [role({ origin: MemberRoleOrigin.INHERITED, scope: PermissionScope.ORGANIZATION })],
        canManage: true,
      }),
    );

    await userEvent.hover(screen.getByText('project-admin'));

    expect(await findTooltip()).toHaveTextContent('organization');
  });

  it('renders a direct and an inherited role side by side', () => {
    render(
      MemberRoleBadges,
      props({
        roles: [
          role({ grantId: 'g1', roleId: 'direct-role' }),
          role({
            grantId: 'g2',
            roleId: 'inherited-role',
            origin: MemberRoleOrigin.INHERITED,
            scope: PermissionScope.ORGANIZATION,
          }),
        ],
        canManage: true,
      }),
    );

    expect(screen.getByText('direct-role')).toBeInTheDocument();
    expect(screen.getByText('inherited-role')).toBeInTheDocument();
    // Only the direct one can be revoked from here.
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });
});
