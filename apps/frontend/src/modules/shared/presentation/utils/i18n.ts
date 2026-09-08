import { i18n, type Messages } from '@lingui/core';

export type SupportedLocale = 'en' | 'fr';

const STORAGE_KEY = 'scylla-locale';

/**
 * Every compiled catalog under `src/modules`, picked up by convention so a new
 * module (or a new locale) is live as soon as `lingui.config.js` knows about it.
 * Listing them by hand is how `core` once ended up extracted but never loaded.
 *
 * Deliberately **not** `{ eager: true }`: eager loading pulled every module's
 * catalog in every locale into the initial chunk. Only the active locale is
 * fetched now, and it is fetched as its own chunk.
 */
const catalogs = import.meta.glob<{ messages: Messages }>('../../../**/locales/*/messages.ts');

const LOCALE_FROM_PATH = /\/locales\/([^/]+)\/messages\.ts$/;

const loadedLocales = new Set<SupportedLocale>();

async function loadCatalogs(locale: SupportedLocale): Promise<void> {
  if (loadedLocales.has(locale)) return;

  const modules = await Promise.all(
    Object.entries(catalogs)
      .filter(([path]) => LOCALE_FROM_PATH.exec(path)?.[1] === locale)
      .map(([, load]) => load()),
  );

  i18n.load(
    locale,
    modules.reduce<Messages>((all, module) => Object.assign(all, module.messages), {}),
  );
  loadedLocales.add(locale);
}

function isSupportedLocale(locale: string | null | undefined): locale is SupportedLocale {
  return locale === 'en' || locale === 'fr';
}

export function getStoredLocale(): SupportedLocale {
  if (typeof window === 'undefined') {
    return 'en';
  }

  const storedLocale = window.localStorage.getItem(STORAGE_KEY);
  if (isSupportedLocale(storedLocale)) {
    return storedLocale;
  }

  const browserLocale = window.navigator.language.split('-')[0];
  return browserLocale === 'fr' ? 'fr' : 'en';
}

/** Loads the locale's catalogs if needed, then activates it. */
export async function setAppLocale(locale: SupportedLocale): Promise<void> {
  await loadCatalogs(locale);
  i18n.activate(locale);

  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale;
  }
}

/**
 * Awaited before the first render (see `main.tsx`), so the app never paints a
 * frame of untranslated text.
 */
export async function initializeAppLocale(): Promise<SupportedLocale> {
  const locale = getStoredLocale();
  await setAppLocale(locale);
  return locale;
}

export function getCurrentLocale(): SupportedLocale {
  return (i18n.locale as SupportedLocale | undefined) ?? 'en';
}
