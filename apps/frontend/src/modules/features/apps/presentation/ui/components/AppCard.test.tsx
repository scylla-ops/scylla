import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AppCard } from './AppCard';
import type { AppEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

const app = (overrides: Partial<AppEntity> = {}): AppEntity => ({
  id: 'app-1',
  organizationId: 'org-1',
  name: 'ci-runner',
  isActive: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  navigateMock.mockClear();
  toastSuccess.mockClear();
});

describe('AppCard', () => {
  it('navigates to the app when the card is clicked', async () => {
    const user = userEvent.setup();
    renderWithI18n(<AppCard app={app()} onRequestDelete={vi.fn()} />);
    await user.click(screen.getByText('ci-runner'));
    expect(navigateMock).toHaveBeenCalledWith('app-1');
  });

  it('shows an "inactive" badge only when the app is not active', () => {
    const { rerender } = renderWithI18n(<AppCard app={app({ isActive: true })} onRequestDelete={vi.fn()} />);
    expect(screen.queryByText('inactive')).not.toBeInTheDocument();

    rerender(
      <I18nProvider i18n={i18n}>
        <AppCard app={app({ isActive: false })} onRequestDelete={vi.fn()} />
      </I18nProvider>,
    );
    expect(screen.getByText('inactive')).toBeInTheDocument();
  });

  it('the footer delete button calls onRequestDelete and does not navigate', async () => {
    const onRequestDelete = vi.fn();
    const user = userEvent.setup();
    const { container } = renderWithI18n(<AppCard app={app()} onRequestDelete={onRequestDelete} />);

    await user.click(container.querySelector('.border-t button')!);

    expect(onRequestDelete).toHaveBeenCalledWith('app-1');
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('copying the id writes the full id to the clipboard and toasts success', async () => {
    const user = userEvent.setup();
    // Defined AFTER userEvent.setup(): user-event installs its own clipboard
    // handling on setup and clobbers a mock defined beforehand.
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });

    const { container } = renderWithI18n(<AppCard app={app({ id: 'app-42' })} onRequestDelete={vi.fn()} />);

    await user.click(container.querySelector('[data-slot="dropdown-menu-trigger"]')!);
    await user.click(screen.getByText('Copy id'));

    expect(writeText).toHaveBeenCalledWith('app-42');
    expect(toastSuccess).toHaveBeenCalledWith('App id copied');
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
