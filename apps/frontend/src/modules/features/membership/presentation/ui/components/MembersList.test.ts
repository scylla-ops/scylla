import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import type { ScopeMember } from '../../../domain/structs/scope-member.struct.ts';
import MembersList from './MembersList.svelte';

const member = (overrides: Partial<ScopeMember> = {}): ScopeMember => ({
  userId: 'user-1',
  roles: [],
  ...overrides,
});

const names: Record<string, string> = {
  'user-1': 'Charlie',
  'user-2': 'Alice',
  'user-3': 'Bob',
};

const baseProps = {
  labelFor: (roleId: string) => roleId,
  canManage: false,
  disabled: false,
  onRevokeRole: vi.fn(),
  addableRolesFor: () => [],
  onAddRole: vi.fn(),
  canRemove: () => true,
  removeTooltip: 'Remove',
  onRemove: vi.fn(),
  nameFor: (id: string) => names[id] ?? id,
  currentUserId: 'user-1',
  members: [] as ScopeMember[],
};

describe('MembersList', () => {
  it('shows a loading spinner and nothing else while loading', () => {
    render(MembersList, { ...baseProps, isLoading: true });

    expect(screen.getByRole('status')).toBeInTheDocument();
    expect(screen.queryByText('Nobody is listed here yet.')).not.toBeInTheDocument();
  });

  it('shows the default empty message with no members', () => {
    render(MembersList, { ...baseProps });
    expect(screen.getByText('Nobody is listed here yet.')).toBeInTheDocument();
  });

  it('shows a custom empty message when given one', () => {
    render(MembersList, { ...baseProps, emptyMessage: 'No admins yet.' });
    expect(screen.getByText('No admins yet.')).toBeInTheDocument();
  });

  it('sorts members by display name rather than the order they were given in', () => {
    render(MembersList, {
      ...baseProps,
      currentUserId: 'none',
      members: [member({ userId: 'user-1' }), member({ userId: 'user-2' }), member({ userId: 'user-3' })],
    });

    const rendered = screen.getAllByText(/Alice|Bob|Charlie/).map(element => element.textContent);
    expect(rendered).toEqual(['Alice', 'Bob', 'Charlie']);
  });

  it("marks the current user's own card and lets other rows be removed", async () => {
    const onRemove = vi.fn();
    render(MembersList, {
      ...baseProps,
      members: [member({ userId: 'user-1' }), member({ userId: 'user-2' })],
      onRemove,
    });

    expect(screen.getByText('You')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button'));

    expect(onRemove).toHaveBeenCalledWith(expect.objectContaining({ userId: 'user-2' }));
  });

  it("forwards addRole with the specific member's userId, not just the roleId", async () => {
    const onAddRole = vi.fn();
    render(MembersList, {
      ...baseProps,
      currentUserId: 'none',
      members: [member({ userId: 'user-2' })],
      canManage: true,
      addableRolesFor: () => [{ roleId: 'r1', name: 'Developer', description: '' }],
      onAddRole,
    });

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'Developer'));

    expect(onAddRole).toHaveBeenCalledWith('user-2', 'r1');
  });

  it('hides the remove action for a member canRemove refuses', () => {
    render(MembersList, {
      ...baseProps,
      currentUserId: 'none',
      members: [member({ userId: 'user-2' })],
      canRemove: () => false,
    });

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
