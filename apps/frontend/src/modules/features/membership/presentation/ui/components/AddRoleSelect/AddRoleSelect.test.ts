import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import AddRoleSelect from './AddRoleSelect.svelte';

const roles = [
  { roleId: 'project-viewer', name: 'Project viewer' },
  { roleId: 'project-admin', name: 'Project admin' },
];

describe('AddRoleSelect', () => {
  it('renders nothing once there is no role left to add', () => {
    render(AddRoleSelect, { roles: [], disabled: false, onSelect: vi.fn() });

    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('offers every addable role', async () => {
    render(AddRoleSelect, { roles, disabled: false, onSelect: vi.fn() });

    await userEvent.click(screen.getByRole('combobox'));

    expect(await findFloating('option', 'Project viewer')).toBeInTheDocument();
    expect(await findFloating('option', 'Project admin')).toBeInTheDocument();
  });

  it('reports the role id that was chosen, not its label', async () => {
    const onSelect = vi.fn();
    render(AddRoleSelect, { roles, disabled: false, onSelect });

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'Project admin'));

    expect(onSelect).toHaveBeenCalledWith('project-admin');
  });

  it('is disabled while a write is in flight', () => {
    render(AddRoleSelect, { roles, disabled: true, onSelect: vi.fn() });

    expect(screen.getByRole('combobox')).toBeDisabled();
  });
});
