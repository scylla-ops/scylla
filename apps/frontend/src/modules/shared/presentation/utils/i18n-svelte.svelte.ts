import { i18n } from '@lingui/core';
import type { MessageDescriptor } from '@lingui/core';

/**
 * Translating from Svelte.
 *
 * `@lingui/core` is already framework-agnostic — only `<Trans>` and `useLingui`
 * are React. What a Svelte component needs on top is reactivity when the locale
 * changes, which is what `locale` below provides: read it inside the template
 * and the component re-runs on activation.
 *
 * **`lingui extract` does not read `.svelte` files.** Declaring a message inside
 * a component would drop it from the catalogs silently — `pnpm i18n:collisions`
 * and the coverage thresholds would both still pass. So messages are declared
 * with the `msg` macro in a neighbouring `*.messages.ts`, which extraction does
 * read, and the component only references them. See `refacto_svelte.md` §4.4.
 */
let locale = $state(i18n.locale);

i18n.on('change', () => {
  locale = i18n.locale;
});

/** The active locale. Reading it in a template is what makes the swap reactive. */
export const activeLocale = (): string => locale;

/**
 * Renders a message descriptor declared in a `*.messages.ts`.
 *
 * Touches `locale` so a component calling it re-renders when the locale changes.
 */
export const t = (descriptor: MessageDescriptor): string => {
  void locale;
  return i18n._(descriptor);
};
