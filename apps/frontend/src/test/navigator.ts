import { vi } from 'vitest';
import { setAppNavigator } from '@platform/context';

/**
 * A fake navigator, installed in place of the router. `navigate(to)` without
 * options records an explicit `undefined`. The query string follows a write, the
 * pathname does not: a path navigation unmounts the caller in the app.
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
    restore: () => setAppNavigator(null),
  };
};
