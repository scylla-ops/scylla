import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { MemberRowAction } from './MemberRowAction';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

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
