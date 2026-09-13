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

beforeEach(() => {
  // Mappers, utils and domain tests opt out of jsdom with
  // `// @vitest-environment node` — there is nothing to stub there.
  if (typeof window === 'undefined') return;

  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
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
