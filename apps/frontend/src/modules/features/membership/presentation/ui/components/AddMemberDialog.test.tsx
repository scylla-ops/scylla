import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AddMemberDialog, type MemberCandidate } from './AddMemberDialog';
import type { AssignableRole } from '@/modules/features/membership/presentation/hooks/use-assignable-roles.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

const candidate = (overrides: Partial<MemberCandidate> = {}): MemberCandidate => ({
  userId: 'user-1',
  username: 'alice',
  ...overrides,
});

const role = (overrides: Partial<AssignableRole> = {}): AssignableRole => ({
  roleId: 'role-1',
  name: 'Developer',
  description: '',
  ...overrides,
});

// Radix's Select needs these to open at all under jsdom.
beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

const openUserPicker = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(screen.getByRole('combobox', { name: /member/i }));
};

describe('AddMemberDialog', () => {
  it('resets to an empty user and the default role each time it opens', () => {
    const { rerender } = renderWithI18n(
      <AddMemberDialog
        open={false}
        onOpenChange={vi.fn()}
        title='Add a member'
        description='...'
        candidates={[candidate()]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[role({ roleId: 'default-role' })]}
        rolesLabel='Roles'
        isPending={false}
        defaultRoleId='default-role'
        onSubmit={vi.fn()}
      />,
    );

    rerender(
      <I18nProvider i18n={i18n}>
        <AddMemberDialog
          open
          onOpenChange={vi.fn()}
          title='Add a member'
          description='...'
          candidates={[candidate()]}
          emptyCandidatesLabel='Nobody left to add'
          roles={[role({ roleId: 'default-role' })]}
          rolesLabel='Roles'
          isPending={false}
          defaultRoleId='default-role'
          onSubmit={vi.fn()}
        />
      </I18nProvider>,
    );

    expect(screen.getByRole('checkbox')).toBeChecked();
  });

  it('disables the user picker and shows the empty-candidates placeholder when there is nobody left', () => {
    renderWithI18n(
      <AddMemberDialog
        open
        onOpenChange={vi.fn()}
        title='Add a member'
        description='...'
        candidates={[]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[]}
        rolesLabel='Roles'
        isPending={false}
        onSubmit={vi.fn()}
      />,
    );

    expect(screen.getByRole('combobox', { name: /member/i })).toBeDisabled();
    expect(screen.getByText('Nobody left to add')).toBeInTheDocument();
  });

  it('the Add button stays disabled until both a user and a role are picked', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <AddMemberDialog
        open
        onOpenChange={vi.fn()}
        title='Add a member'
        description='...'
        candidates={[candidate()]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[role()]}
        rolesLabel='Roles'
        isPending={false}
        onSubmit={vi.fn()}
      />,
    );

    const addButton = screen.getByRole('button', { name: 'Add' });
    expect(addButton).toBeDisabled();

    await user.click(screen.getByRole('checkbox'));
    expect(addButton).toBeDisabled(); // still no user picked

    await openUserPicker(user);
    await user.click(await screen.findByText('alice'));
    expect(addButton).toBeEnabled();
  });

  it('submitting calls onSubmit with the picked user and roles, and closes on success', async () => {
    const onOpenChange = vi.fn();
    const onSubmit = vi.fn().mockResolvedValue(true);
    const user = userEvent.setup();
    renderWithI18n(
      <AddMemberDialog
        open
        onOpenChange={onOpenChange}
        title='Add a member'
        description='...'
        candidates={[candidate({ userId: 'user-9', username: 'bob' })]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[role({ roleId: 'role-9' })]}
        rolesLabel='Roles'
        isPending={false}
        onSubmit={onSubmit}
      />,
    );

    await user.click(screen.getByRole('checkbox'));
    await openUserPicker(user);
    await user.click(await screen.findByText('bob'));
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(onSubmit).toHaveBeenCalledWith('user-9', ['role-9']);
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it('does not close the dialog when onSubmit resolves false', async () => {
    const onOpenChange = vi.fn();
    const onSubmit = vi.fn().mockResolvedValue(false);
    const user = userEvent.setup();
    renderWithI18n(
      <AddMemberDialog
        open
        onOpenChange={onOpenChange}
        title='Add a member'
        description='...'
        candidates={[candidate()]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[role()]}
        rolesLabel='Roles'
        isPending={false}
        onSubmit={onSubmit}
      />,
    );

    await user.click(screen.getByRole('checkbox'));
    await openUserPicker(user);
    await user.click(await screen.findByText('alice'));
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(onSubmit).toHaveBeenCalled();
    expect(onOpenChange).not.toHaveBeenCalled();
  });

  it('Cancel closes the dialog without submitting', async () => {
    const onOpenChange = vi.fn();
    const onSubmit = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <AddMemberDialog
        open
        onOpenChange={onOpenChange}
        title='Add a member'
        description='...'
        candidates={[candidate()]}
        emptyCandidatesLabel='Nobody left to add'
        roles={[role()]}
        rolesLabel='Roles'
        isPending={false}
        onSubmit={onSubmit}
      />,
    );

    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onSubmit).not.toHaveBeenCalled();
  });
});
