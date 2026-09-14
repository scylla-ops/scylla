import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { messages } from '@/modules/features/membership/locales/fr/messages.ts';
import { renderWithI18n } from '@/test/render.tsx';
import { withLocale } from '@/test/i18n.ts';
import { MemberCard } from './MemberCard';
import {
  MemberRoleOrigin,
  type MemberRole,
} from '@/modules/features/membership/domain/structs/scope-member.struct.ts';
import { PermissionScope } from '@platform/authz';

/**
 * `_0` is an exact-match plural arm, not a CLDR category — it has to survive
 * extraction and compilation intact, and French wording it differently from
 * the `one` arm ("Aucun rôle" vs "1 rôle") is precisely where it would break.
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
};

describe('MemberCard role count in French', () => {
  it('uses the exact-zero arm rather than the plural one', () => {
    renderWithI18n(<MemberCard {...baseProps} roles={[]} />);
    expect(screen.getAllByText('Aucun rôle').length).toBeGreaterThan(0);
  });

  it('uses the singular arm, substituting the count for #', () => {
    renderWithI18n(<MemberCard {...baseProps} roles={[role()]} />);
    expect(screen.getByText('1 rôle')).toBeInTheDocument();
  });

  it('uses the plural arm above one', () => {
    renderWithI18n(
      <MemberCard {...baseProps} roles={[role(), role({ grantId: 'grant-2' })]} />,
    );
    expect(screen.getByText('2 rôles')).toBeInTheDocument();
  });
});
