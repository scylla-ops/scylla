import { i18n, type Messages } from '@lingui/core';
import { afterEach, beforeEach } from 'vitest';

/** Activates a real compiled catalog for a test file, to check a translation actually renders. */
export const withLocale = (locale: string, messages: Messages) => {
  i18n.load(locale, messages);

  // Per test: the restore below runs after every test.
  beforeEach(() => {
    i18n.activate(locale);
  });

  afterEach(() => {
    i18n.activate('en');
  });
};
