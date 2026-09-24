import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import type { AssignableRole } from '../../../assignable-roles.state.svelte.ts';
import AddMemberDialog from './AddMemberDialog.svelte';

const assignable = (overrides: Partial<AssignableRole> = {}): AssignableRole => ({
  roleId: 'project-viewer',
  name: 'Project viewer',
  description: 'Read-only access',
  ...overrides,
});

const props = (overrides: Record<string, unknown> = {}) => ({
  open: true,
  onOpenChange: vi.fn(),
  title: 'Add a member',
  description: 'Admit someone to this project.',
  candidates: [
    { userId: 'user-1', username: 'alice' },
    { userId: 'user-2', username: 'bob' },
  ],
  emptyCandidatesLabel: 'Everyone is already a member',
  roles: [assignable(), assignable({ roleId: 'project-admin', name: 'Project admin' })],
  rolesLabel: 'Roles',
  isPending: false,
  onSubmit: vi.fn().mockResolvedValue(true),
  ...overrides,
});

const pickUser = async (username: string) => {
  await userEvent.click(screen.getByRole('combobox'));
  await userEvent.click(await findFloating('option', username));
};

describe('AddMemberDialog', () => {
  it('shows its title and description', () => {
    render(AddMemberDialog, props());

    expect(screen.getByText('Add a member')).toBeInTheDocument();
    expect(screen.getByText('Admit someone to this project.')).toBeInTheDocument();
  });

  it('cannot submit before a user and a role are chosen', () => {
    render(AddMemberDialog, props());

    expect(screen.getByRole('button', { name: /add/i })).toBeDisabled();
  });

  it('pre-ticks the floor role, which is what belonging to the scope means', () => {
    render(AddMemberDialog, props({ defaultRoleId: 'project-viewer' }));

    const [viewer, admin] = screen.getAllByRole('checkbox');
    expect(viewer).toBeChecked();
    expect(admin).not.toBeChecked();
  });

  it('still refuses to submit with a role but no user', () => {
    render(AddMemberDialog, props({ defaultRoleId: 'project-viewer' }));

    expect(screen.getByRole('button', { name: /add/i })).toBeDisabled();
  });

  it('submits the chosen user with every ticked role', async () => {
    const onSubmit = vi.fn().mockResolvedValue(true);
    render(AddMemberDialog, props({ defaultRoleId: 'project-viewer', onSubmit }));

    await pickUser('bob');
    await userEvent.click(screen.getAllByRole('checkbox')[1]);
    await userEvent.click(screen.getByRole('button', { name: /add/i }));

    expect(onSubmit).toHaveBeenCalledWith('user-2', ['project-viewer', 'project-admin']);
  });

  it('closes once the member is actually in', async () => {
    const onOpenChange = vi.fn();
    render(
      AddMemberDialog,
      props({
        defaultRoleId: 'project-viewer',
        onOpenChange,
        onSubmit: vi.fn().mockResolvedValue(true),
      }),
    );

    await pickUser('alice');
    await userEvent.click(screen.getByRole('button', { name: /add/i }));

    await vi.waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
  });

  it('stays open when the grant was refused, so the selection is not lost', async () => {
    const onOpenChange = vi.fn();
    render(
      AddMemberDialog,
      props({
        defaultRoleId: 'project-viewer',
        onOpenChange,
        onSubmit: vi.fn().mockResolvedValue(false),
      }),
    );

    await pickUser('alice');
    await userEvent.click(screen.getByRole('button', { name: /add/i }));

    expect(onOpenChange).not.toHaveBeenCalledWith(false);
  });

  it('cancel closes without submitting', async () => {
    const onOpenChange = vi.fn();
    const onSubmit = vi.fn();
    render(AddMemberDialog, props({ onOpenChange, onSubmit }));

    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('explains the empty picker rather than offering an empty list', () => {
    render(AddMemberDialog, props({ candidates: [] }));

    expect(screen.getByText('Everyone is already a member')).toBeInTheDocument();
    expect(screen.getByRole('combobox')).toBeDisabled();
  });

  it('locks the whole form while the grant is in flight', () => {
    render(AddMemberDialog, props({ isPending: true, defaultRoleId: 'project-viewer' }));

    expect(screen.getByRole('combobox')).toBeDisabled();
    expect(screen.getByRole('button', { name: /add/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
  });

  it('says when the role list is still loading', () => {
    render(AddMemberDialog, props({ roles: [], rolesLoading: true }));

    expect(screen.getByText('Loading…')).toBeInTheDocument();
  });
});
