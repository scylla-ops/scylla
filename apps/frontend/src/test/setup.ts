import '@testing-library/jest-dom/vitest';
import { i18n } from '@lingui/core';
import { beforeEach, vi } from 'vitest';

// An empty catalog: lingui falls back to the message id, the English source. `test/i18n.ts` loads a real one.
i18n.load('en', {});
i18n.activate('en');

/** Browser APIs jsdom lacks, stubbed per test (a test's teardown may strip them). A test overrides one in its own `beforeEach`. */
class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

/** A Svelte transition needs `element.animate()`. The stub keeps running, so a test can see the leaving state; call `finish()` to end it. */
class AnimationStub {
  currentTime = 0;
  startTime = 0;
  playState = 'running';
  onfinish: (() => void) | null = null;
  onremove: (() => void) | null = null;
  finished = new Promise<void>(() => {});
  effect = { getComputedTiming: () => ({ duration: 0 }), setKeyframes: () => {} };

  pause = () => (this.playState = 'paused');
  play = () => (this.playState = 'running');
  cancel = () => (this.playState = 'idle');
  reverse = vi.fn();
  commitStyles = vi.fn();
  addEventListener = vi.fn();
  removeEventListener = vi.fn();

  finish = () => {
    this.playState = 'finished';
    this.onfinish?.();
  };
}

beforeEach(() => {
  // Mappers, utils and domain tests opt out of jsdom with
  // `// @vitest-environment node` — there is nothing to stub there.
  if (typeof window === 'undefined') return;

  // bits-ui can leave `pointer-events: none` on the body after a dialog; it would block the next test's clicks.
  document.body.style.pointerEvents = '';

  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  Element.prototype.animate = vi.fn(() => new AnimationStub() as unknown as Animation);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
  // jsdom logs "Not implemented" for it.
  window.scrollTo = vi.fn();
  // No media queries in jsdom: report the desktop layout.
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
});
