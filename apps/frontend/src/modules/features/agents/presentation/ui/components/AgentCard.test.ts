import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { contextStore } from '@platform/context';
import { PermissionScope, permissionsStore } from '@platform/authz';
import type { AgentEntity } from '../../../domain/entities/agent.entity.ts';
import AgentCard from './AgentCard.svelte';

const toastSuccess = vi.fn();
vi.mock('svelte-sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

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

const fullControl = () =>
  permissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

let testNavigator: ReturnType<typeof installTestNavigator>;

beforeEach(() => {
  toastSuccess.mockClear();
  testNavigator = installTestNavigator({ pathname: '/acme/agents' });
  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  fullControl();
});

afterEach(() => testNavigator.restore());

describe('AgentCard', () => {
  it('navigates to the agent when the card is clicked', async () => {
    render(AgentCard, { agent: agent(), onRequestDelete: vi.fn() });

    await userEvent.click(screen.getByText('runner-1'));

    expect(testNavigator.navigate).toHaveBeenCalledWith('/acme/agents/agent-1', {});
  });

  it('shows "online" for a connected agent, "offline" for a disconnected one', async () => {
    const { rerender, unmount } = render(AgentCard, {
      agent: agent({ connected: true }),
      onRequestDelete: vi.fn(),
    });
    expect(screen.getByText('online')).toBeInTheDocument();

    await rerender({ agent: agent({ connected: false }) });
    expect(screen.getByText('offline')).toBeInTheDocument();
    unmount();
  });

  it('says "seen <time ago>" when connected, "down <time ago>" when not, given a lastSeen', async () => {
    const { rerender } = render(AgentCard, {
      agent: agent({ connected: true, lastSeen: '2026-01-01T00:00:00.000Z' }),
      onRequestDelete: vi.fn(),
    });
    expect(screen.getByText(/seen/)).toBeInTheDocument();

    await rerender({ agent: agent({ connected: false, lastSeen: '2026-01-01T00:00:00.000Z' }) });
    expect(screen.getByText(/down/)).toBeInTheDocument();
  });

  it('says "never connected" when there is no lastSeen at all', () => {
    render(AgentCard, { agent: agent({ lastSeen: '' }), onRequestDelete: vi.fn() });
    expect(screen.getByText('never connected')).toBeInTheDocument();
  });

  it('the footer delete button calls onRequestDelete and does not navigate', async () => {
    const onRequestDelete = vi.fn();
    render(AgentCard, { agent: agent(), onRequestDelete });

    // Findable by name now: the port gave the icon-only control an `sr-only`
    // label, where the React test had to reach for `.border-t button`.
    await userEvent.click(screen.getByRole('button', { name: 'Delete agent' }));

    expect(onRequestDelete).toHaveBeenCalledWith('agent-1');
    expect(testNavigator.navigate).not.toHaveBeenCalled();
  });

  it('hides both delete affordances without DELETE_APP', async () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(AgentCard, { agent: agent(), onRequestDelete: vi.fn() });

    expect(screen.queryByRole('button', { name: 'Delete agent' })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Agent actions' }));
    await expect(findFloating('menuitem', 'Delete')).rejects.toThrow();
  });

  it('copying the id writes the full id to the clipboard and toasts success', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal('navigator', { ...window.navigator, clipboard: { writeText } });

    render(AgentCard, { agent: agent({ id: 'agent-42' }), onRequestDelete: vi.fn() });

    await userEvent.click(screen.getByRole('button', { name: 'Agent actions' }));
    await userEvent.click(await findFloating('menuitem', 'Copy id'));

    expect(writeText).toHaveBeenCalledWith('agent-42');
    expect(toastSuccess).toHaveBeenCalledWith('Agent id copied');
    expect(testNavigator.navigate).not.toHaveBeenCalled();
  });
});
