import { on } from 'svelte/events';

/** The URL of the page, reactive. The router writes it; nothing else does. */
export const location = $state({ pathname: '/', search: '', hash: '' });

/** Copies the URL of the document into `location`. */
export const syncLocation = (): void => {
  location.pathname = window.location.pathname;
  location.search = window.location.search;
  location.hash = window.location.hash;
};

/** Changes the URL without a page load, then updates `location`. */
export const changeLocation = (url: string, options: { replace?: boolean } = {}): void => {
  if (options.replace) history.replaceState(history.state, '', url);
  else history.pushState(null, '', url);
  syncLocation();
};

const isInternalLink = (event: MouseEvent, anchor: HTMLAnchorElement): boolean =>
  event.button === 0 &&
  !event.defaultPrevented &&
  !(event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) &&
  anchor.hasAttribute('href') &&
  !anchor.hasAttribute('download') &&
  (!anchor.target || anchor.target === '_self') &&
  anchor.origin === window.location.origin;

const followLink = (event: MouseEvent): void => {
  const anchor = (event.target as Element | null)?.closest('a');
  if (!(anchor instanceof HTMLAnchorElement) || !isInternalLink(event, anchor)) return;

  const samePage =
    anchor.pathname === window.location.pathname && anchor.search === window.location.search;
  if (samePage && anchor.hash) return;

  event.preventDefault();
  changeLocation(anchor.pathname + anchor.search + anchor.hash);
  if (!samePage) window.scrollTo(0, 0);
};

/**
 * Keeps `location` in step with the browser: back and forward, and a click on a
 * link of the app, which it follows without a page load. Returns the function
 * that stops it.
 */
export const listenToLocation = (): (() => void) => {
  syncLocation();
  const stops = [on(window, 'popstate', syncLocation), on(window, 'click', followLink)];
  return () => stops.forEach(stop => stop());
};
