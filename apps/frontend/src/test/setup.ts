import '@testing-library/jest-dom/vitest';
import { i18n } from '@lingui/core';
import { beforeEach, vi } from 'vitest';

// The app itself loads compiled catalogs and activates a locale in App.tsx
// (see modules/core/presentation/ui/App.tsx). Tests don't render that entry
// point, so `t`/`Trans` need at least a loaded+activated locale to resolve —
// an empty catalog is enough: lingui falls back to the message id, which
// happens to be the source English text for every macro call in this
// codebase, and loading (even empty) silences its "not loaded" warning.
//
// A test that needs a *real* translation (see modules/shared/test/i18n.ts)
// loads its catalog itself and restores this default afterwards.
i18n.load('en', {});
i18n.activate('en');

/**
 * Browser APIs jsdom doesn't implement, stubbed once for the whole suite.
 *
 * Radix reaches for all of these on its own (Tooltip and Popover measure
 * themselves, Select captures the pointer and scrolls the active item into
 * view), so any test rendering a shadcn control needs them — which was 57
 * copies of this block across the suite before it moved here. None of them
 * carries test-specific meaning: they only keep jsdom from throwing.
 *
 * They are re-applied per test because `vi.restoreAllMocks()` / `unstubAllGlobals`
 * in a test's own teardown would otherwise strip them for the next one.
 *
 * A test that needs to *drive* one of these — asserting on a real resize, say —
 * overrides it in its own `beforeEach`, which runs after this one.
 */
class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

class IntersectionObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn(() => []);
}

/**
 * jsdom implements no Web Animations API, and a Svelte `transition:` runs on
 * `element.animate()` — without this, every component with one throws
 * "element.animate is not a function" the moment it enters or leaves.
 *
 * The stub stays *running* rather than resolving: a transition that finished
 * instantly would tear its node down before a test could observe the leaving
 * state, which is precisely what the page transitions are about. Tests that
 * care about the end of an animation call `finish()` on the returned handle.
 */
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

  // bits-ui locks the page behind an open dialog with an inline
  // `pointer-events: none` on `<body>`, and unmounting the component during
  // testing-library's cleanup does not always get to restore it. The stale lock
  // then makes `userEvent` refuse every click in the *next* test of the file,
  // with an error that points at the innocent test. Clearing it at the start of
  // each test is unambiguous: no test begins with a dialog already open.
  document.body.style.pointerEvents = '';

  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
  vi.stubGlobal('IntersectionObserver', IntersectionObserverStub);
  Element.prototype.animate = vi.fn(() => new AnimationStub() as unknown as Animation);
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
  // jsdom logs "Not implemented: Window's scrollTo()" instead of throwing;
  // noise, not a failure, but it drowns real output.
  window.scrollTo = vi.fn();
  // jsdom has no media queries at all. Report "no match", i.e. the desktop
  // layout — the shadcn sidebar asks for `max-width: 767px` to decide whether
  // to render as a drawer.
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
