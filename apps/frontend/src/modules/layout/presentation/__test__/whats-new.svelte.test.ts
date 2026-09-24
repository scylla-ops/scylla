import { describe, it, expect, beforeEach } from 'vitest';
import { flushSync } from 'svelte';
import { WHATS_NEW } from '@/modules/layout/whats-new.ts';
import {
  dismissRelease,
  highlightIdForNav,
  isUnseen,
  markSeen,
  unseenRelease,
} from '../whats-new.svelte.ts';

beforeEach(() => {
  localStorage.clear();
});

describe('isUnseen', () => {
  it('is true before anything is marked seen', () => {
    expect(isUnseen('dashboard')).toBe(true);
  });

  it('is false once markSeen is called for that id', () => {
    markSeen('dashboard');
    expect(isUnseen('dashboard')).toBe(false);
  });

  it('marking one id seen does not affect another', () => {
    markSeen('dashboard');
    expect(isUnseen('roles')).toBe(true);
  });

  it('reads the seen flag of the current release version', () => {
    localStorage.setItem(`whats-new-seen:${WHATS_NEW.version}:dashboard`, '1');
    expect(isUnseen('dashboard')).toBe(false);
  });

  it('updates a reactive reader when an id is marked seen', () => {
    const cleanup = $effect.root(() => {
      const unseen = $derived(isUnseen('dashboard'));
      flushSync();
      expect(unseen).toBe(true);

      markSeen('dashboard');
      flushSync();

      expect(unseen).toBe(false);
    });
    cleanup();
  });
});

describe('highlightIdForNav', () => {
  it('finds the highlight id for a nav url this release announces', () => {
    expect(highlightIdForNav('dashboard')).toBe('dashboard');
  });

  it('returns undefined for a url with no highlight in this release', () => {
    expect(highlightIdForNav('some-unrelated-page')).toBeUndefined();
  });
});

describe('the release announcement', () => {
  it('is the current release while the user did not dismiss it', () => {
    expect(unseenRelease()).toBe(WHATS_NEW);
  });

  it('is null after the user dismisses it', () => {
    dismissRelease();
    expect(unseenRelease()).toBeNull();
  });
});
