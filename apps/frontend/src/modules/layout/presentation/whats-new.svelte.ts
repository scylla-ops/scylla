import { WHATS_NEW, type Release } from '../whats-new.ts';

const RELEASE_ID = 'release';

const seenKey = (id: string) => `whats-new-seen:${WHATS_NEW.version}:${id}`;

let revision = $state(0);

export const markSeen = (id: string): void => {
  localStorage.setItem(seenKey(id), '1');
  revision += 1;
};

/** Reactive. */
export const isUnseen = (id: string): boolean => {
  void revision;
  return localStorage.getItem(seenKey(id)) === null;
};

export const highlightIdForNav = (navUrl: string): string | undefined =>
  WHATS_NEW.highlights.find(highlight => highlight.navUrl === navUrl)?.id;

/** Reactive. */
export const unseenRelease = (): Release | null => (isUnseen(RELEASE_ID) ? WHATS_NEW : null);

export const dismissRelease = (): void => markSeen(RELEASE_ID);
