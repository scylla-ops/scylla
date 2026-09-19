import { vi } from 'vitest';
import { setAppNavigator } from '@platform/context';

/**
 * A navigator for a test that exercises navigation.
 *
 * `scyllaNavigate` and everything built on it go through the singleton in
 * `@platform/context`, which the shell installs at startup — a test never
 * renders the shell, so it installs this instead. That replaces the
 * `vi.mock('react-router-dom', …)` those tests used to carry, and it works the
 * same for a React component and a Svelte one, which is the point.
 *
 * ```ts
 * const nav = installTestNavigator({ pathname: '/acme/projects/p1' });
 * afterEach(nav.restore);
 * expect(nav.navigate).toHaveBeenCalledWith('/acme/projects/p1/secrets', {});
 * ```
 *
 * Note the second argument: the navigator contract is `navigate(to, options?)`,
 * so a call made without options records an explicit `undefined`.
 */
export const installTestNavigator = (options: { pathname?: string } = {}) => {
  const navigate = vi.fn();
  const back = vi.fn();
  const pathname = options.pathname ?? '/';

  setAppNavigator({ navigate, back, pathname: () => pathname });

  return {
    navigate,
    back,
    /** Module state, so a test that installs one has to take it back out. */
    restore: () => setAppNavigator(null),
  };
};
