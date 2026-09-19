/**
 * The one thing in the app that knows how to change the URL.
 *
 * `useNavigate` is a React hook, and a Svelte page has no React tree to call it
 * from. Rather than teach every island to reach back across the bridge, the
 * shell registers its router here once and everything else — hooks, view models,
 * `useScyllaNavigate` — goes through these functions.
 *
 * Same shape as `theme.store.ts`: an agnostic core plus a binding per framework.
 * It is also what makes the Phase 6 router swap a one-file change, because the
 * only code that names react-router is the registration in `Core.router.tsx`.
 */

export interface NavigateOptions {
  replace?: boolean;
}

/** What the shell's router has to provide. Deliberately tiny. */
export interface AppNavigator {
  navigate: (to: string, options?: NavigateOptions) => void;
  back: () => void;
  /** A function, not a value: the pathname changes under the same navigator. */
  pathname: () => string;
}

let current: AppNavigator | null = null;

/**
 * Installs the router. Called by the composition root, and by a test that
 * drives navigation — pass `null` to uninstall it again.
 */
export const setAppNavigator = (navigator: AppNavigator | null): void => {
  current = navigator;
};

const require = (): AppNavigator => {
  // Throwing beats a silent no-op: a navigation that quietly does nothing looks
  // like a backend failure, and this can only ever be a wiring mistake.
  if (!current) {
    throw new Error('No navigator installed — the shell must call setAppNavigator() first.');
  }
  return current;
};

export const navigateTo = (to: string, options?: NavigateOptions): void =>
  require().navigate(to, options);

export const navigateBack = (): void => require().back();

/**
 * The path currently displayed.
 *
 * Falls back to `window.location` so a caller that only *reads* the URL — to
 * build a sub-route, to highlight the active nav entry — works before the shell
 * has registered anything, which is the common case in a component test.
 */
export const currentPathname = (): string =>
  current ? current.pathname() : window.location.pathname;
