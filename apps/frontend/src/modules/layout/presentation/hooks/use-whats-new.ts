import { useSyncExternalStore } from 'react';
import { WHATS_NEW } from '@/modules/layout/whats-new.ts';

/** The announcement of the release itself, as opposed to one of its highlights. */
const RELEASE_ID = 'release';

/** Scoped to the version, so bumping it re-arms every announcement at once. */
const seenKey = (id: string) => `whats-new-seen:${WHATS_NEW.version}:${id}`;

// localStorage is the source of truth and nothing mirrors it into state: every
// badge for the same id re-reads it when one of them is dismissed.
const listeners = new Set<() => void>();

const subscribe = (notify: () => void) => {
  listeners.add(notify);
  return () => {
    listeners.delete(notify);
  };
};

export const markSeen = (id: string) => {
  localStorage.setItem(seenKey(id), '1');
  listeners.forEach(notify => notify());
};

/** True while this announcement hasn't been dismissed in the current release. */
export const useIsUnseen = (id: string) =>
  useSyncExternalStore(subscribe, () => localStorage.getItem(seenKey(id)) === null);

/** The highlight a sidebar entry announces, if this release has one for it. */
export const highlightIdForNav = (navUrl: string) =>
  WHATS_NEW.highlights.find(highlight => highlight.navUrl === navUrl)?.id;

/** The release to announce, until the user acknowledges it. */
export const useUnseenRelease = () => {
  const isUnseen = useIsUnseen(RELEASE_ID);

  return {
    release: isUnseen ? WHATS_NEW : null,
    dismiss: () => markSeen(RELEASE_ID),
  };
};
