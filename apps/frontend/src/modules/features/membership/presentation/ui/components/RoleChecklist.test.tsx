import { describe, it, expect, vi } from 'vitest';
import { render as renderRTL, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RoleChecklist } from './RoleChecklist';
import type { AssignableRole } from '@/modules/features/membership/presentation/hooks/use-assignable-roles.ts';

const render = (ui: React.ReactElement) => renderRTL(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const role = (overrides: Partial<AssignableRole> = {}): AssignableRole => ({
  roleId: 'role-1',
  name: 'Developer',
  description: '',
  ...overrides,
});

describe('RoleChecklist', () => {
  it('shows a loading placeholder instead of the list', () => {
    render(<RoleChecklist label='Roles' roles={[]} isLoading selected={new Set()} onToggle={vi.fn()} />);
    expect(screen.getByText('Loading…')).toBeInTheDocument();
    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
  });

  it('shows an empty placeholder when there is nothing to grant', () => {
    render(<RoleChecklist label='Roles' roles={[]} selected={new Set()} onToggle={vi.fn()} />);
    expect(screen.getByText('No role can be granted here.')).toBeInTheDocument();
  });

  it('renders each role name and, when present, its description', () => {
    render(
      <RoleChecklist
        label='Roles'
        roles={[role({ roleId: 'a', name: 'Admin', description: 'Full access' }), role({ roleId: 'b', name: 'Viewer' })]}
        selected={new Set()}
        onToggle={vi.fn()}
      />,
    );
    expect(screen.getByText('Admin')).toBeInTheDocument();
    expect(screen.getByText('Full access')).toBeInTheDocument();
    expect(screen.getByText('Viewer')).toBeInTheDocument();
  });

  it('checks the boxes for roles already in the selected set', () => {
    render(
      <RoleChecklist
        label='Roles'
        roles={[role({ roleId: 'a' }), role({ roleId: 'b' })]}
        selected={new Set(['b'])}
        onToggle={vi.fn()}
      />,
    );
    const [first, second] = screen.getAllByRole('checkbox');
    expect(first).not.toBeChecked();
    expect(second).toBeChecked();
  });

  it('clicking an unheld role toggles it', async () => {
    const onToggle = vi.fn();
    const user = userEvent.setup();
    render(<RoleChecklist label='Roles' roles={[role({ roleId: 'x' })]} selected={new Set()} onToggle={onToggle} />);

    await user.click(screen.getByRole('checkbox'));
    expect(onToggle).toHaveBeenCalledWith('x');
  });

  it('a role already held is checked, disabled, labelled "Held", and ignores clicks', async () => {
    const onToggle = vi.fn();
    const user = userEvent.setup();
    render(
      <RoleChecklist
        label='Roles'
        roles={[role({ roleId: 'x' })]}
        selected={new Set()}
        alreadyHeld={new Set(['x'])}
        onToggle={onToggle}
      />,
    );

    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeChecked();
    expect(checkbox).toBeDisabled();
    expect(screen.getByText('Held')).toBeInTheDocument();

    await user.click(checkbox);
    expect(onToggle).not.toHaveBeenCalled();
  });

  it('the disabled prop disables every checkbox regardless of held state', () => {
    render(
      <RoleChecklist label='Roles' roles={[role({ roleId: 'x' })]} selected={new Set()} disabled onToggle={vi.fn()} />,
    );
    expect(screen.getByRole('checkbox')).toBeDisabled();
  });
});
