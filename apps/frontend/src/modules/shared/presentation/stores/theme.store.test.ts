import { describe, it, expect, beforeEach, vi } from 'vitest';
import { getTheme, setTheme, subscribeToTheme } from './theme.store.ts';

beforeEach(() => {
  setTheme('dark');
  localStorage.clear();
});

describe('theme store', () => {
  it('puts the class on <html>, which is what every `dark:` utility reads', () => {
    setTheme('light');
    expect(document.documentElement.classList.contains('dark')).toBe(false);

    setTheme('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  it('persists under the key `index.html` reads before the first paint', () => {
    setTheme('light');

    expect(localStorage.getItem('scylla-theme')).toBe('light');
  });

  it('notifies subscribers on a change', () => {
    const listener = vi.fn();
    subscribeToTheme(listener);

    setTheme('light');

    expect(listener).toHaveBeenCalledTimes(1);
    expect(getTheme()).toBe('light');
  });

  it('stays silent when the theme is set to the one already applied', () => {
    const listener = vi.fn();
    subscribeToTheme(listener);

    setTheme('dark');

    expect(listener).not.toHaveBeenCalled();
  });

  it('stops notifying once unsubscribed', () => {
    const listener = vi.fn();
    const unsubscribe = subscribeToTheme(listener);

    unsubscribe();
    setTheme('light');

    expect(listener).not.toHaveBeenCalled();
  });

  it('leaves no transition-killing style behind once the swap is done', () => {
    setTheme('light');

    // The <style> exists only for the frame the class swap takes; a leaked one
    // would freeze every transition in the app.
    expect(document.head.querySelector('style')).toBeNull();
  });
});
