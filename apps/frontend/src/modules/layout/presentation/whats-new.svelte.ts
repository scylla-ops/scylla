import { WHATS_NEW, type Release } from '../whats-new.ts';

const RELEASE_ID = 'release';

const seenKey = (id: string) => `whats-new-seen:${WHATS_NEW.version}:${id}`;

let revision = $state(0);

/** Records that the user saw this announcement. */
export const markSeen = (id: string): void => {
  localStorage.setItem(seenKey(id), '1');
  revision += 1;
};

/** True while the user did not dismiss this announcement in the current release. Reactive. */
export const isUnseen = (id: string): boolean => {
  void revision;
  return localStorage.getItem(seenKey(id)) === null;
};

/** The highlight that a sidebar entry announces, if this release has one for it. */
export const highlightIdForNav = (navUrl: string): string | undefined =>
  WHATS_NEW.highlights.find(highlight => highlight.navUrl === navUrl)?.id;

/** The release to announce, until the user dismisses it. Reactive. */
export const unseenRelease = (): Release | null => (isUnseen(RELEASE_ID) ? WHATS_NEW : null);

/** Dismisses the release announcement. */
export const dismissRelease = (): void => markSeen(RELEASE_ID);
