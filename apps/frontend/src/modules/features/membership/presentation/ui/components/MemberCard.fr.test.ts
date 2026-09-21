import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import { messages } from '../../../locales/fr/messages.ts';
import { render } from '@/test/render.svelte.ts';
import { withLocale } from '@/test/i18n.ts';
import { PermissionScope } from '@platform/authz';
import {
  MemberRoleOrigin,
  type MemberRole,
} from '../../../domain/structs/scope-member.struct.ts';
import MemberCard from './MemberCard.svelte';

/**
 * The exact-zero arm is not a CLDR category — it has to survive extraction and
 * compilation intact, and French wording it differently from the `one` arm
 * ("Aucun rôle" vs "1 rôle") is precisely where it would break.
 *
 * It is also what pins the port: the React `<Plural _0=…>` became
 * `plural(value, { 0: … })`, and if those two did *not* compile to the same
 * msgid the French would silently fall back to English here.
 */
withLocale('fr', messages);

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
  removeTooltip: 'Retirer',
  onRemove: vi.fn(),
  roles: [] as MemberRole[],
};

describe('MemberCard role count in French', () => {
  it('uses the exact-zero arm rather than the plural one', () => {
    render(MemberCard, { ...baseProps });
    expect(screen.getAllByText('Aucun rôle').length).toBeGreaterThan(0);
  });

  it('uses the singular arm, substituting the count for #', () => {
    render(MemberCard, { ...baseProps, roles: [role()] });
    expect(screen.getByText('1 rôle')).toBeInTheDocument();
  });

  it('uses the plural arm above one', () => {
    render(MemberCard, { ...baseProps, roles: [role(), role({ grantId: 'grant-2' })] });
    expect(screen.getByText('2 rôles')).toBeInTheDocument();
  });
});
