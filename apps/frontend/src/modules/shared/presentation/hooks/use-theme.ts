import { useSyncExternalStore } from 'react';
import {
  getTheme,
  setTheme,
  subscribeToTheme,
  type Theme,
} from '@shared/presentation/stores/theme.store.ts';

interface UseThemeResult {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

/**
 * React binding over the theme store. The store itself is framework-agnostic —
 * this hook is the only part of it React needs, and the only part that goes
 * away once the UI is Svelte.
 *
 * There is no `system` theme: the app picks dark by default and the user's
 * choice is explicit, so `theme` is always the one actually applied.
 */
export const useTheme = (): UseThemeResult => ({
  theme: useSyncExternalStore(subscribeToTheme, getTheme, getTheme),
  setTheme,
});
