import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { RoleDetailPermissions } from './RoleDetailPermissions';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import { Permission, PermissionScope } from '@platform/authz';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Developer',
  description: '',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

describe('RoleDetailPermissions', () => {
  it('says the role grants full control', () => {
    renderWithI18n(<RoleDetailPermissions role={role({ access: { kind: 'fullControl' } })} />);
    expect(screen.getByText('Grants full control over its scope.')).toBeInTheDocument();
  });

  it('says a restricted role with nothing ticked has no permissions', () => {
    renderWithI18n(
      <RoleDetailPermissions role={role({ access: { kind: 'restricted', permissions: [] } })} />,
    );
    expect(screen.getByText('No permissions.')).toBeInTheDocument();
  });

  it('lists each permission of a restricted role as a badge', () => {
    renderWithI18n(
      <RoleDetailPermissions
        role={role({
          access: { kind: 'restricted', permissions: [Permission.READ_PROJECT, Permission.DELETE_JOB] },
        })}
      />,
    );
    // Exact labels come from usePermissionLabels, tested on its own - this only
    // checks two badges rendered, not zero and not one merged string.
    expect(document.querySelectorAll('.font-normal').length).toBeGreaterThanOrEqual(2);
  });

  it('falls back to an "unknown access" message for an access kind newer than this build', () => {
    renderWithI18n(<RoleDetailPermissions role={role({ access: { kind: 'unknown' } })} />);
    expect(screen.getByText('Unknown access.')).toBeInTheDocument();
  });
});
