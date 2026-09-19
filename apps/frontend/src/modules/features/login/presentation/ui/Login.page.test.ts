import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withRegistry, withQueryClient } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';
import LoginPage from './Login.page.svelte';

/**
 * What `use-login.test.tsx` and `LoginForm.test.tsx` used to cover between them,
 * now driven end to end through the page: the form, the mutation and the
 * redirect are one view model, and the repository behind it is injected through
 * the DI registry rather than mocked.
 */

const makeFakeRepository = (overrides: Partial<LoginRepository> = {}) => {
  const login = overrides.login ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  return { repository: { login } satisfies LoginRepository, login };
};

let teardown: Array<() => void> = [];
let testNavigator: ReturnType<typeof installTestNavigator>;

const setUp = (repository: LoginRepository) => {
  const cache = withQueryClient();
  const restoreRegistry = withRegistry({ login: { loginRepository: repository } });
  testNavigator = installTestNavigator();
  teardown = [cache.restore, restoreRegistry, testNavigator.restore];
};

beforeEach(() => {
  teardown = [];
});

afterEach(() => teardown.forEach(restore => restore()));

const signIn = async (username = 'ravenne', password = 'hunter2') => {
  const user = userEvent.setup();
  await user.type(screen.getByLabelText('Username'), username);
  await user.type(screen.getByLabelText('Password'), password);
  await user.click(screen.getByRole('button', { name: 'Login' }));
};

describe('LoginPage', () => {
  it('signs in with the typed credentials and redirects to the root, replacing history', async () => {
    const { repository, login } = makeFakeRepository();
    setUp(repository);
    render(LoginPage);

    await signIn();

    await waitFor(() => expect(login).toHaveBeenCalledWith('ravenne', 'hunter2'));
    await waitFor(() =>
      expect(testNavigator.navigate).toHaveBeenCalledWith('/', { replace: true }),
    );
  });

  it('stays on the page when the credentials are refused', async () => {
    const error = new ScyllaError('bad credentials', { cause: { code: 'INVALID_CREDENTIALS' } });
    const { repository } = makeFakeRepository({
      login: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    setUp(repository);
    render(LoginPage);

    await signIn('ravenne', 'wrong');

    await waitFor(() => expect(screen.getByRole('button', { name: 'Login' })).toBeEnabled());
    expect(testNavigator.navigate).not.toHaveBeenCalled();
    // The form is still there to try again — no loading screen took it over.
    expect(screen.getByLabelText('Username')).toBeInTheDocument();
  });

  it('replaces the form with the loading screen once sign-in succeeds', async () => {
    const { repository } = makeFakeRepository();
    setUp(repository);
    render(LoginPage);

    await signIn();

    await waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Login' })).not.toBeInTheDocument(),
    );
  });
});
