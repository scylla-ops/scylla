import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { useContextStore } from '@platform/context';
import type { AppEntity } from '../../../domain/entities/app.entity.ts';
import AppCard from './AppCard.svelte';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({ toast: { success: (message: string) => toastSuccess(message) } }));

const app = (overrides: Partial<AppEntity> = {}): AppEntity => ({
  id: 'app-1',
  organizationId: 'org-1',
  name: 'ci-runner',
  isActive: true,
  createdAt: '2026-01-05T10:00:00Z',
  updatedAt: '2026-01-05T10:00:00Z',
  ...overrides,
});

let testNavigator: ReturnType<typeof installTestNavigator>;

beforeEach(() => {
  toastSuccess.mockClear();
  testNavigator = installTestNavigator({ pathname: '/acme/apps' });
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
});

afterEach(() => testNavigator.restore());

describe('AppCard', () => {
  it('opens the app when the card is clicked', async () => {
    render(AppCard, { app: app(), onRequestDelete: vi.fn() });

    await userEvent.click(screen.getByText('ci-runner'));

    expect(testNavigator.navigate).toHaveBeenCalledWith('/acme/apps/app-1', {});
  });

  it('shows an inactive badge only for a disabled app', () => {
    const { unmount } = render(AppCard, { app: app(), onRequestDelete: vi.fn() });
    expect(screen.queryByText('inactive')).not.toBeInTheDocument();
    unmount();

    render(AppCard, { app: app({ isActive: false }), onRequestDelete: vi.fn() });
    expect(screen.getByText('inactive')).toBeInTheDocument();
  });

  it('asks for deletion without navigating into the app', async () => {
    const onRequestDelete = vi.fn();
    render(AppCard, { app: app(), onRequestDelete });

    await userEvent.click(screen.getByRole('button', { name: 'Delete app' }));

    expect(onRequestDelete).toHaveBeenCalledWith('app-1');
    // The row click must not fire too — that is what `stopPropagation` buys.
    expect(testNavigator.navigate).not.toHaveBeenCalled();
  });

  it('disables both delete affordances without DELETE_APP', async () => {
    render(AppCard, { app: app(), onRequestDelete: vi.fn(), canDelete: false });

    expect(screen.getByRole('button', { name: 'Delete app' })).toBeDisabled();

    await userEvent.click(screen.getByRole('button', { name: 'App actions' }));
    // `findFloating`, not `findByRole`: bits-ui puts the menu in a floating
    // layer that floating-ui leaves `visibility: hidden` under jsdom, which
    // hides the whole subtree from the accessibility tree.
    expect(await findFloating('menuitem', 'Delete')).toHaveAttribute('data-disabled');
  });

  it('copies the id from the menu and says so', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal('navigator', { ...window.navigator, clipboard: { writeText } });

    render(AppCard, { app: app(), onRequestDelete: vi.fn() });

    await userEvent.click(screen.getByRole('button', { name: 'App actions' }));
    await userEvent.click(await findFloating('menuitem', 'Copy id'));

    expect(writeText).toHaveBeenCalledWith('app-1');
    expect(toastSuccess).toHaveBeenCalledWith('App id copied');
  });
});
