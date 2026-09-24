/** The only way to change the URL. The shell installs the router with `setAppNavigator`. */

export interface NavigateOptions {
  replace?: boolean;
}

export interface AppNavigator {
  navigate: (to: string, options?: NavigateOptions) => void;
  back: () => void;
  pathname: () => string;
  /** With its leading `?`. */
  search: () => string;
}

let current: AppNavigator | null = null;

/** `null` uninstalls it (tests). */
export const setAppNavigator = (navigator: AppNavigator | null): void => {
  current = navigator;
};

const require = (): AppNavigator => {
  // Throw: a navigation that silently does nothing can only be a wiring mistake.
  if (!current) {
    throw new Error('No navigator installed — the shell must call setAppNavigator() first.');
  }
  return current;
};

export const navigateTo = (to: string, options?: NavigateOptions): void =>
  require().navigate(to, options);

export const navigateBack = (): void => require().back();

/** Falls back to `window.location` when no router is installed, as in a component test. */
export const currentPathname = (): string =>
  current ? current.pathname() : window.location.pathname;

export const currentSearch = (): string =>
  current ? current.search() : window.location.search;
