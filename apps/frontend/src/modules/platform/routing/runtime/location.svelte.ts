import { on } from 'svelte/events';

/** Written only by the router. */
export const location = $state({ pathname: '/', search: '', hash: '' });

export const syncLocation = (): void => {
  location.pathname = window.location.pathname;
  location.search = window.location.search;
  location.hash = window.location.hash;
};

/** Changes the URL without a page load. */
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

/** Follows back/forward and the clicks on the app's links. Returns the function that stops it. */
export const listenToLocation = (): (() => void) => {
  syncLocation();
  const stops = [on(window, 'popstate', syncLocation), on(window, 'click', followLink)];
  return () => stops.forEach(stop => stop());
};
