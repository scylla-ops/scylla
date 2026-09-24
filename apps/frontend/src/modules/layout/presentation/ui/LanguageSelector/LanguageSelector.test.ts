import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import InSidebar from '../__test__/InSidebar.fixture.svelte';
import LanguageSelector from './LanguageSelector.svelte';

const setAppLocale = vi.fn((_locale: string) => Promise.resolve());
vi.mock('@shared/presentation/utils/i18n.ts', () => ({
  setAppLocale: (locale: string) => setAppLocale(locale),
}));

beforeEach(() => vi.clearAllMocks());

describe('LanguageSelector', () => {
  it('shows the active locale', () => {
    render(InSidebar, { component: LanguageSelector });

    expect(screen.getByRole('button', { name: /Language/ })).toHaveTextContent('EN');
  });

  it('switches the locale of the app', async () => {
    render(InSidebar, { component: LanguageSelector });
    const user = userEvent.setup();

    await user.click(screen.getByRole('button', { name: /Language/ }));
    await user.click(await findFloating('menuitemradio', 'Français'));

    expect(setAppLocale).toHaveBeenCalledWith('fr');
  });
});
