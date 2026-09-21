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
 *
 * **The query string follows a write; the pathname does not.** A page that keeps
 * state in the query rewrites it and reads it straight back — the job details
 * page and its log panels — so a spy that only recorded the call would make that
 * round trip untestable. A *path* navigation is the opposite case: in the app it
 * unmounts the component that made it, so a fixed pathname is the honest model,
 * and moving it would make a test that clicks two links in one render build the
 * second URL on top of the first.
 */
export const installTestNavigator = (options: { pathname?: string; search?: string } = {}) => {
  const pathname = options.pathname ?? '/';
  let search = options.search ?? '';

  const navigate = vi.fn((to: string) => {
    const index = to.indexOf('?');
    search = index === -1 ? '' : to.slice(index);
  });
  const back = vi.fn();

  setAppNavigator({
    navigate,
    back,
    pathname: () => pathname,
    search: () => search,
  });

  return {
    navigate,
    back,
    /** Module state, so a test that installs one has to take it back out. */
    restore: () => setAppNavigator(null),
  };
};
