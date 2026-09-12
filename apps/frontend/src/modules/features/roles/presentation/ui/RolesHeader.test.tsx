import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RolesHeader } from './RolesHeader';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

describe('RolesHeader', () => {
  it('shows the role count', () => {
    renderWithI18n(<RolesHeader count={3} />);
    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByText('Roles')).toBeInTheDocument();
  });

  it('hides "Create role" entirely without onNew', () => {
    renderWithI18n(<RolesHeader count={0} />);
    expect(screen.queryByRole('button', { name: 'Create role' })).not.toBeInTheDocument();
  });

  it('"Create role" calls onNew when given', async () => {
    const onNew = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RolesHeader count={0} onNew={onNew} />);
    await user.click(screen.getByRole('button', { name: 'Create role' }));
    expect(onNew).toHaveBeenCalled();
  });

  it('forwards the selection props through to FeatureHeader', async () => {
    const onSelectAll = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<RolesHeader count={5} onSelectAll={onSelectAll} />);
    await user.click(screen.getByRole('button', { name: 'Select all' }));
    expect(onSelectAll).toHaveBeenCalled();
  });
});
