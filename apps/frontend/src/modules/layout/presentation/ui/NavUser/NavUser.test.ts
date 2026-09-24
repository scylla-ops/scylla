import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { contextStore } from '@platform/context';
import { installTestNavigator } from '@/test/navigator.ts';
import { findFloating, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import InSidebar from '../__test__/InSidebar.fixture.svelte';
import NavUser from './NavUser.svelte';

const signOut = vi.fn();
vi.mock('../../sign-out.ts', () => ({ signOut: () => signOut() }));

let teardown: Array<() => void> = [];
let navigator: ReturnType<typeof installTestNavigator>;

const setUp = (getById: () => Promise<unknown>) => {
  const cache = withQueryClient();
  teardown = [cache.restore, withRegistry({ user: { userRepository: { getById } } })];
  return render(InSidebar, { component: NavUser });
};

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.setItem('userId', 'user-1');
  navigator = installTestNavigator();
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme' } });
});

afterEach(() => {
  teardown.forEach(restore => restore());
  navigator.restore();
  localStorage.clear();
});

describe('NavUser', () => {
  it('shows a loading label until the user arrives', () => {
    setUp(() => new Promise(() => {}));

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows the name and the initial of the user', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success({ userId: 'user-1', username: 'ravenne' })));

    expect(await screen.findByText('ravenne')).toBeInTheDocument();
    expect(screen.getByText('R')).toBeInTheDocument();
  });

  it('opens the settings of the user from the menu', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success({ userId: 'user-1', username: 'ravenne' })));
    const user = userEvent.setup();

    await user.click(await screen.findByRole('button', { name: /ravenne/ }));
    await user.click(await findFloating('menuitem', 'Settings'));

    expect(navigator.navigate).toHaveBeenCalledWith('/acme/users/user-1', { replace: true });
  });

  it('signs the user out from the menu', async () => {
    setUp(() => Promise.resolve(ScyllaResult.success({ userId: 'user-1', username: 'ravenne' })));
    const user = userEvent.setup();

    await user.click(await screen.findByRole('button', { name: /ravenne/ }));
    await user.click(await findFloating('menuitem', 'Log out'));

    expect(signOut).toHaveBeenCalled();
  });
});
