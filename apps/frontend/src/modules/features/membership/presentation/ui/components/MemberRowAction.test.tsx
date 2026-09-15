import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { MemberRowAction } from './MemberRowAction';

describe('MemberRowAction', () => {
  it('shows a "You" badge for the current user, regardless of canRemove', () => {
    renderWithI18n(
      <MemberRowAction isCurrentUser canRemove tooltip='Remove' onRemove={vi.fn()} />,
    );
    expect(screen.getByText('You')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders nothing when the member cannot be removed', () => {
    const { container } = renderWithI18n(
      <MemberRowAction isCurrentUser={false} canRemove={false} tooltip='Remove' onRemove={vi.fn()} />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('a removable member gets a trash button that calls onRemove', async () => {
    const onRemove = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <MemberRowAction isCurrentUser={false} canRemove tooltip='Remove' onRemove={onRemove} />,
    );

    await user.click(screen.getByRole('button'));
    expect(onRemove).toHaveBeenCalled();
  });

  it('the remove button respects disabled', () => {
    renderWithI18n(
      <MemberRowAction isCurrentUser={false} canRemove disabled tooltip='Remove' onRemove={vi.fn()} />,
    );
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
