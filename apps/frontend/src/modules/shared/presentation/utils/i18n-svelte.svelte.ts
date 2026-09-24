import { i18n } from '@lingui/core';
import type { MessageDescriptor } from '@lingui/core';

/**
 * `lingui extract` does not read `.svelte` files: declare messages with `msg` in
 * a `*.messages.ts` and render them with `t`, which re-renders on a locale switch.
 */
let locale = $state(i18n.locale);

i18n.on('change', () => {
  locale = i18n.locale;
});

/** Read it in a template to re-render on a locale switch. */
export const activeLocale = (): string => locale;

export const t = (descriptor: MessageDescriptor): string => {
  void locale;
  return i18n._(descriptor);
};
