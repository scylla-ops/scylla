import { describe, it, expect, beforeEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { markSeen, useIsUnseen, useUnseenRelease, highlightIdForNav } from './use-whats-new';
import { WHATS_NEW } from '@/modules/layout/whats-new.ts';

beforeEach(() => {
  localStorage.clear();
});

describe('useIsUnseen', () => {
  it('is true before anything has been marked seen', () => {
    const { result } = renderHook(() => useIsUnseen('dashboard'));
    expect(result.current).toBe(true);
  });

  it('flips to false once markSeen is called for that id', () => {
    const { result } = renderHook(() => useIsUnseen('dashboard'));

    act(() => markSeen('dashboard'));

    expect(result.current).toBe(false);
  });

  it('marking one id seen does not affect another', () => {
    const { result } = renderHook(() => useIsUnseen('roles'));

    act(() => markSeen('dashboard'));

    expect(result.current).toBe(true);
  });

  it('is scoped by the current release version (a stale seen-flag from another version would not exist under this key anyway)', () => {
    localStorage.setItem(`whats-new-seen:${WHATS_NEW.version}:dashboard`, '1');
    const { result } = renderHook(() => useIsUnseen('dashboard'));
    expect(result.current).toBe(false);
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

describe('useUnseenRelease', () => {
  it('exposes the release while it has not been dismissed', () => {
    const { result } = renderHook(() => useUnseenRelease());
    expect(result.current.release).toBe(WHATS_NEW);
  });

  it('dismiss() marks the whole release seen, and release becomes null', () => {
    const { result } = renderHook(() => useUnseenRelease());

    act(() => result.current.dismiss());

    expect(result.current.release).toBeNull();
  });
});
