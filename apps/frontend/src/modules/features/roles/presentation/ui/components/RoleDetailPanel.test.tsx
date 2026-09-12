import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RoleDetailPanel } from './RoleDetailPanel';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import { PermissionScope } from '@platform/authz';

vi.mock(
  '@/modules/features/roles/presentation/ui/components/role-detail/RoleDetailHeader.tsx',
  () => ({
    RoleDetailHeader: ({ role }: { role: RoleEntity }) => <div data-testid='header'>{role.name}</div>,
  }),
);
vi.mock(
  '@/modules/features/roles/presentation/ui/components/role-detail/RoleDetailPermissions.tsx',
  () => ({
    default: ({ role }: { role: RoleEntity }) => <div data-testid='permissions'>{role.id}</div>,
  }),
);
vi.mock(
  '@/modules/features/roles/presentation/ui/components/role-detail/RoleDetailGrantList.tsx',
  () => ({
    default: ({ role }: { role: RoleEntity }) => <div data-testid='grants'>{role.id}</div>,
  }),
);

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

describe('RoleDetailPanel', () => {
  it('shows a placeholder when no role is selected', () => {
    renderWithI18n(<RoleDetailPanel role={null} onEdit={vi.fn()} />);
    expect(screen.getByText('Select a role to see its permissions and members.')).toBeInTheDocument();
    expect(screen.queryByTestId('header')).not.toBeInTheDocument();
  });

  it('composes the header, permissions and grant list for the selected role', () => {
    renderWithI18n(<RoleDetailPanel role={role} onEdit={vi.fn()} />);
    expect(screen.getByTestId('header')).toHaveTextContent('Developer');
    expect(screen.getByTestId('permissions')).toHaveTextContent('role-1');
    expect(screen.getByTestId('grants')).toHaveTextContent('role-1');
  });
});
