import { createSubscriber } from 'svelte/reactivity';

export type Theme = 'light' | 'dark';

/** Shared with the inline script in `index.html` — the two must agree. */
const STORAGE_KEY = 'scylla-theme';

/**
 * `index.html` puts the class on `<html>` before the first paint, reading the
 * same key. The DOM is therefore already correct by the time this module loads,
 * and reading it back is what keeps the two from drifting apart.
 *
 * The `document` guard is for the tests that opt out of jsdom: this module
 * touches the DOM at import time, so a pure test importing it transitively
 * would otherwise throw before reaching its first assertion.
 */
const themeInDocument = (): Theme => {
  if (typeof document === 'undefined') return 'dark';
  return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
};

let current: Theme = themeInDocument();

const listeners = new Set<() => void>();

export const getTheme = (): Theme => current;

/** Calls `listener` after each change of the theme. Returns the function that stops it. */
export const subscribeToTheme = (listener: () => void): (() => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/**
 * Swapping the class animates every `transition-colors` in the tree at once,
 * which reads as the page smearing. Suppressing transitions for the one frame
 * the swap takes is what `next-themes`' `disableTransitionOnChange` did.
 */
const withoutTransitions = (swap: () => void): void => {
  const style = document.createElement('style');
  style.append(document.createTextNode('*,*::before,*::after{transition:none!important}'));
  document.head.appendChild(style);

  swap();

  // Measuring forces the reflow that commits the swap before transitions are
  // allowed back — without it the browser batches both and animates anyway.
  document.body.getBoundingClientRect();

  document.head.removeChild(style);
};

export const setTheme = (theme: Theme): void => {
  if (theme === current) return;

  current = theme;

  if (typeof document !== 'undefined') {
    withoutTransitions(() => {
      document.documentElement.classList.toggle('dark', theme === 'dark');
    });
    localStorage.setItem(STORAGE_KEY, theme);
  }

  listeners.forEach(listener => listener());
};

const trackTheme = createSubscriber(update => subscribeToTheme(update));

/** The current theme. Reactive: a Svelte component that reads it updates when the theme changes. */
export const currentTheme = (): Theme => {
  trackTheme();
  return current;
};
