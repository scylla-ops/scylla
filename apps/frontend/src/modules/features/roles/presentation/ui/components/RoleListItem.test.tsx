import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RoleListItem } from './RoleListItem';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import { PermissionScope } from '@platform/authz';

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

describe('RoleListItem', () => {
  it('shows the role name, description fallback, and member count', () => {
    renderWithI18n(
      <RoleListItem
        role={role({ description: '' })}
        memberCount={4}
        active={false}
        selected={false}
        selectable
        onOpen={vi.fn()}
        onToggleSelect={vi.fn()}
      />,
    );
    expect(screen.getByText('Developer')).toBeInTheDocument();
    expect(screen.getByText('No description')).toBeInTheDocument();
    expect(screen.getByText('4 members')).toBeInTheDocument();
  });

  it('clicking the card calls onOpen', async () => {
    const onOpen = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <RoleListItem
        role={role()}
        memberCount={0}
        active={false}
        selected={false}
        selectable
        onOpen={onOpen}
        onToggleSelect={vi.fn()}
      />,
    );
    await user.click(screen.getByText('Developer'));
    expect(onOpen).toHaveBeenCalled();
  });

  it('clicking the checkbox toggles selection without opening the role', async () => {
    const onOpen = vi.fn();
    const onToggleSelect = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <RoleListItem
        role={role()}
        memberCount={0}
        active={false}
        selected={false}
        selectable
        onOpen={onOpen}
        onToggleSelect={onToggleSelect}
      />,
    );
    await user.click(screen.getByRole('checkbox'));
    expect(onToggleSelect).toHaveBeenCalled();
    expect(onOpen).not.toHaveBeenCalled();
  });

  it('the checkbox is disabled when the role is not selectable', () => {
    renderWithI18n(
      <RoleListItem
        role={role()}
        memberCount={0}
        active={false}
        selected={false}
        selectable={false}
        onOpen={vi.fn()}
        onToggleSelect={vi.fn()}
      />,
    );
    expect(screen.getByRole('checkbox')).toBeDisabled();
  });
});
