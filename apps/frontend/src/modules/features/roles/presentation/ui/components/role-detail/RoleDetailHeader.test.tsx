import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { usePermissionsStore, PermissionScope } from '@platform/authz';
import { RoleDetailHeader } from './RoleDetailHeader';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Developer',
  description: '',
  scope: PermissionScope.PROJECT,
  origin: { kind: 'custom' },
  access: { kind: 'fullControl' },
  ...overrides,
});

beforeEach(() => {
  usePermissionsStore.setState({
    permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
  });
});

describe('RoleDetailHeader', () => {
  it('shows the name, description fallback, and a "Full control" badge for a full-control role', () => {
    renderWithI18n(<RoleDetailHeader role={role({ description: '' })} onEdit={vi.fn()} />);
    expect(screen.getByText('Developer')).toBeInTheDocument();
    expect(screen.getByText('No description')).toBeInTheDocument();
    expect(screen.getByText('Full control')).toBeInTheDocument();
  });

  it('omits the "Full control" badge for a restricted role', () => {
    renderWithI18n(
      <RoleDetailHeader role={role({ access: { kind: 'restricted', permissions: [] } })} onEdit={vi.fn()} />,
    );
    expect(screen.queryByText('Full control')).not.toBeInTheDocument();
  });

  it.each([
    [{ kind: 'builtin', key: 'admin' }, 'Built-in'],
    [{ kind: 'custom' }, 'Custom'],
    [{ kind: 'unknown' }, 'Unknown'],
  ] as const)('labels a %o origin as "%s"', (origin, label) => {
    renderWithI18n(<RoleDetailHeader role={role({ origin })} onEdit={vi.fn()} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it('a built-in role cannot be edited', () => {
    renderWithI18n(<RoleDetailHeader role={role({ origin: { kind: 'builtin', key: 'admin' } })} onEdit={vi.fn()} />);
    expect(screen.getByRole('button', { name: /edit/i })).toBeDisabled();
  });

  it('Edit calls onEdit with the role, when permitted', async () => {
    const onEdit = vi.fn();
    const user = userEvent.setup();
    const theRole = role();
    renderWithI18n(<RoleDetailHeader role={theRole} onEdit={onEdit} />);

    await user.click(screen.getByRole('button', { name: /edit/i }));
    expect(onEdit).toHaveBeenCalledWith(theRole);
  });

  it('Edit is disabled without MANAGE_ROLES', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    renderWithI18n(<RoleDetailHeader role={role()} onEdit={vi.fn()} />);
    expect(screen.getByRole('button', { name: /edit/i })).toBeDisabled();
  });
});
