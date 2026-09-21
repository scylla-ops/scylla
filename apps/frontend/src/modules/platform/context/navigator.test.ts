import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  currentPathname,
  currentSearch,
  navigateBack,
  navigateTo,
  setAppNavigator,
  type AppNavigator,
} from './navigator.ts';

const fake = (pathname = '/acme', search = '') => ({
  navigate: vi.fn<AppNavigator['navigate']>(),
  back: vi.fn<AppNavigator['back']>(),
  pathname: () => pathname,
  search: () => search,
});

afterEach(() => setAppNavigator(null));

describe('the app navigator', () => {
  it('forwards a navigation to the installed router, options included', () => {
    const navigator = fake();
    setAppNavigator(navigator);

    navigateTo('/acme/projects', { replace: true });

    expect(navigator.navigate).toHaveBeenCalledWith('/acme/projects', { replace: true });
  });

  it('throws rather than silently doing nothing when no router is installed', () => {
    expect(() => navigateTo('/acme')).toThrow(/setAppNavigator/);
    expect(() => navigateBack()).toThrow(/setAppNavigator/);
  });

  it('reads the pathname through the router, so it follows navigation', () => {
    setAppNavigator(fake('/acme/projects/project-1'));
    expect(currentPathname()).toBe('/acme/projects/project-1');
  });

  it('reads the query string through the router too, so URL-backed state follows it', () => {
    setAppNavigator(fake('/acme/projects/p1/jobs/j1', '?nodes=build,test'));
    expect(currentSearch()).toBe('?nodes=build,test');
  });

  it('falls back to the document location when nothing is installed', () => {
    // A component test that only *reads* the path must not have to install a
    // router, which is what makes this fallback worth having.
    expect(currentPathname()).toBe(window.location.pathname);
    expect(currentSearch()).toBe(window.location.search);
  });

  it('uninstalling makes the next read fall back again', () => {
    setAppNavigator(fake('/acme'));
    setAppNavigator(null);
    expect(currentPathname()).toBe(window.location.pathname);
  });
});
