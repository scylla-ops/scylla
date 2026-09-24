import { createSubscriber } from 'svelte/reactivity';

export type Theme = 'light' | 'dark';

/** Shared with the inline script in `index.html`. */
const STORAGE_KEY = 'scylla-theme';

/**
 * `index.html` sets the class before the first paint: read it back from the DOM.
 * The `document` guard is for the tests that run without jsdom.
 */
const themeInDocument = (): Theme => {
  if (typeof document === 'undefined') return 'dark';
  return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
};

let current: Theme = themeInDocument();

const listeners = new Set<() => void>();

export const getTheme = (): Theme => current;

export const subscribeToTheme = (listener: () => void): (() => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/** Disables transitions for the frame of the class swap, or every color animates at once. */
const withoutTransitions = (swap: () => void): void => {
  const style = document.createElement('style');
  style.append(document.createTextNode('*,*::before,*::after{transition:none!important}'));
  document.head.appendChild(style);

  swap();

  // Force the reflow, or the browser batches both changes and animates anyway.
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

/** Reactive. */
export const currentTheme = (): Theme => {
  trackTheme();
  return current;
};
