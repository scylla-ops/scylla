import { i18n, type Messages } from '@lingui/core';

export type SupportedLocale = 'en' | 'fr';

const STORAGE_KEY = 'scylla-locale';

/** Every compiled catalog, found by convention. Lazy: only the active locale is loaded. */
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

export async function setAppLocale(locale: SupportedLocale): Promise<void> {
  await loadCatalogs(locale);
  i18n.activate(locale);

  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale;
  }
}

/** Awaited before the first render, so no frame shows untranslated text. */
export async function initializeAppLocale(): Promise<SupportedLocale> {
  const locale = getStoredLocale();
  await setAppLocale(locale);
  return locale;
}

export function getCurrentLocale(): SupportedLocale {
  return (i18n.locale as SupportedLocale | undefined) ?? 'en';
}
