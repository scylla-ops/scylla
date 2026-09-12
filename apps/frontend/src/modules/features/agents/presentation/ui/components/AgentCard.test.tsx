import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { AgentCard } from './AgentCard';
import type { AgentEntity } from '@/modules/features/agents/domain/entities/agent.entity.ts';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

const agent = (overrides: Partial<AgentEntity> = {}): AgentEntity => ({
  id: 'agent-1',
  organizationId: 'org-1',
  name: 'runner-1',
  isActive: true,
  connected: true,
  lastSeen: '2026-01-01T00:00:00.000Z',
  inFlight: 0,
  host: null,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  navigateMock.mockClear();
  toastSuccess.mockClear();
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

describe('AgentCard', () => {
  it('navigates to the agent when the card is clicked', async () => {
    const user = userEvent.setup();
    renderWithI18n(<AgentCard agent={agent()} onRequestDelete={vi.fn()} />);
    await user.click(screen.getByText('runner-1'));
    expect(navigateMock).toHaveBeenCalledWith('agent-1');
  });

  it('shows "online" for a connected agent, "offline" for a disconnected one', () => {
    const { rerender } = renderWithI18n(<AgentCard agent={agent({ connected: true })} onRequestDelete={vi.fn()} />);
    expect(screen.getByText('online')).toBeInTheDocument();

    rerender(
      <I18nProvider i18n={i18n}>
        <AgentCard agent={agent({ connected: false })} onRequestDelete={vi.fn()} />
      </I18nProvider>,
    );
    expect(screen.getByText('offline')).toBeInTheDocument();
  });

  it('says "seen <time ago>" when connected, "down <time ago>" when not, given a lastSeen', () => {
    const { rerender } = renderWithI18n(
      <AgentCard agent={agent({ connected: true, lastSeen: '2026-01-01T00:00:00.000Z' })} onRequestDelete={vi.fn()} />,
    );
    expect(screen.getByText(/seen/)).toBeInTheDocument();

    rerender(
      <I18nProvider i18n={i18n}>
        <AgentCard agent={agent({ connected: false, lastSeen: '2026-01-01T00:00:00.000Z' })} onRequestDelete={vi.fn()} />
      </I18nProvider>,
    );
    expect(screen.getByText(/down/)).toBeInTheDocument();
  });

  it('says "never connected" when there is no lastSeen at all', () => {
    renderWithI18n(<AgentCard agent={agent({ lastSeen: '' })} onRequestDelete={vi.fn()} />);
    expect(screen.getByText('never connected')).toBeInTheDocument();
  });

  it('the footer delete button calls onRequestDelete and does not navigate', async () => {
    const onRequestDelete = vi.fn();
    const user = userEvent.setup();
    const { container } = renderWithI18n(<AgentCard agent={agent()} onRequestDelete={onRequestDelete} />);

    // Two delete-capable buttons exist (menu item + footer icon button); the
    // footer one is the plain icon button, found by elimination via its class.
    const footerDeleteBtn = container.querySelector('.border-t button')!;
    await user.click(footerDeleteBtn);

    expect(onRequestDelete).toHaveBeenCalledWith('agent-1');
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('copying the id writes the full id to the clipboard and toasts success', async () => {
    const user = userEvent.setup();
    // Defined AFTER userEvent.setup(): user-event installs its own clipboard
    // handling on setup and clobbers a mock defined beforehand.
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });

    const { container } = renderWithI18n(<AgentCard agent={agent({ id: 'agent-42' })} onRequestDelete={vi.fn()} />);

    await user.click(container.querySelector('[data-slot="dropdown-menu-trigger"]')!);
    await user.click(screen.getByText('Copy id'));

    expect(writeText).toHaveBeenCalledWith('agent-42');
    expect(toastSuccess).toHaveBeenCalledWith('Agent id copied');
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
