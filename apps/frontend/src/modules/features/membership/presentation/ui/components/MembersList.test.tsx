import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MembersList } from './MembersList';
import type { ScopeMember } from '@/modules/features/membership/domain/structs/scope-member.struct.ts';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

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

const baseMetadata = {
  labelFor: (roleId: string) => roleId,
  canManage: false,
  disabled: false,
  onRevokeRole: vi.fn(),
  addableRolesFor: () => [],
  onAddRole: vi.fn(),
  canRemove: () => true,
  removeTooltip: 'Remove',
  onRemove: vi.fn(),
};

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

describe('MembersList', () => {
  it('shows a loading spinner and nothing else while loading', () => {
    const { container } = renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[]}
        nameFor={id => names[id] ?? id}
        currentUserId='user-1'
        isLoading
      />,
    );
    expect(container.querySelector('.animate-spin')).toBeInTheDocument();
    expect(screen.queryByText('Nobody is listed here yet.')).not.toBeInTheDocument();
  });

  it('shows the default empty message with no members', () => {
    renderWithI18n(
      <MembersList {...baseMetadata} members={[]} nameFor={id => names[id] ?? id} currentUserId='user-1' />,
    );
    expect(screen.getByText('Nobody is listed here yet.')).toBeInTheDocument();
  });

  it('shows a custom empty message when given one', () => {
    renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[]}
        nameFor={id => names[id] ?? id}
        currentUserId='user-1'
        emptyMessage='No admins yet.'
      />,
    );
    expect(screen.getByText('No admins yet.')).toBeInTheDocument();
  });

  it('sorts members by display name rather than the order they were given in', () => {
    renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[member({ userId: 'user-1' }), member({ userId: 'user-2' }), member({ userId: 'user-3' })]}
        nameFor={id => names[id] ?? id}
        currentUserId='none'
      />,
    );
    const renderedNames = screen.getAllByText(/Alice|Bob|Charlie/).map(el => el.textContent);
    expect(renderedNames).toEqual(['Alice', 'Bob', 'Charlie']);
  });

  it("marks the current user's own card and lets other rows be removed", async () => {
    const onRemove = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[member({ userId: 'user-1' }), member({ userId: 'user-2' })]}
        nameFor={id => names[id] ?? id}
        currentUserId='user-1'
        onRemove={onRemove}
      />,
    );

    expect(screen.getByText('You')).toBeInTheDocument();
    await user.click(screen.getByRole('button'));
    expect(onRemove).toHaveBeenCalledWith(expect.objectContaining({ userId: 'user-2' }));
  });

  it('forwards addRole with the specific member\'s userId, not just the roleId', async () => {
    const onAddRole = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[member({ userId: 'user-2' })]}
        nameFor={id => names[id] ?? id}
        currentUserId='none'
        canManage
        addableRolesFor={() => [{ roleId: 'r1', name: 'Developer', description: '' }]}
        onAddRole={onAddRole}
      />,
    );

    await user.click(screen.getByRole('combobox'));
    await user.click(await screen.findByText('Developer'));
    expect(onAddRole).toHaveBeenCalledWith('user-2', 'r1');
  });

  it("hides the remove action for a member canRemove refuses", () => {
    renderWithI18n(
      <MembersList
        {...baseMetadata}
        members={[member({ userId: 'user-2' })]}
        nameFor={id => names[id] ?? id}
        currentUserId='none'
        canRemove={() => false}
      />,
    );
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
