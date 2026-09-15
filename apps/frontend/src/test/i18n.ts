import { i18n, type Messages } from '@lingui/core';
import { afterEach, beforeEach } from 'vitest';

/**
 * Activates a real compiled catalog for the duration of a test file.
 *
 * `setup.ts` activates English with an *empty* catalog, which makes every
 * message fall back to its id — good enough for the ~150 files that assert on
 * English copy, but it means nothing in the suite ever proves a translation
 * actually renders. `pnpm i18n:collisions` catches two catalogs disagreeing on
 * a msgid; it cannot catch a French plural or interpolation that breaks at
 * runtime. That is what this is for.
 *
 * The catalogs are compiled output (`pnpm compile`, part of `prebuild` and run
 * by CI before the tests), keyed by a hash of message + context. The SWC macro
 * rewrites `<Trans>` call sites to the same hash, so loading the catalog is all
 * it takes — no mapping by hand.
 */
export const withLocale = (locale: string, messages: Messages) => {
  i18n.load(locale, messages);

  // Per test, not once per file: the restore below runs after every test, so
  // activating at module scope would only ever translate the first one.
  beforeEach(() => {
    i18n.activate(locale);
  });

  afterEach(() => {
    // Back to the suite-wide default, or every later file in this worker would
    // inherit the locale.
    i18n.activate('en');
  });
};
