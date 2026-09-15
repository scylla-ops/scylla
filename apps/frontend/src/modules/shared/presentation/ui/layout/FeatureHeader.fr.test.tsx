import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { messages } from '@/modules/shared/locales/fr/messages.ts';
import { renderWithI18n } from '@/test/render.tsx';
import { withLocale } from '@/test/i18n.ts';
import { FeatureHeader } from './FeatureHeader';

/**
 * The one place the suite renders a real translation rather than the English
 * fallback. Plurals and interpolations are where a catalog actually breaks —
 * a missing `#`, a plural form the target language needs and English doesn't —
 * and `i18n:collisions` can't see any of it.
 */
const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

withLocale('fr', messages);

beforeEach(() => {
  toastSuccess.mockClear();
});

const getDeleteButton = (): HTMLElement => {
  const button = document.querySelector('button[data-variant="destructive"]');
  if (!button) throw new Error('delete button not found');
  return button as HTMLElement;
};

describe('FeatureHeader in French', () => {
  it('interpolates the label into the New button', () => {
    renderWithI18n(<FeatureHeader label='pipeline' onNew={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'Nouveau pipeline' })).toBeInTheDocument();
  });

  it('picks the singular plural form, substituting the count for #', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader
        label='pipeline'
        selectedCount={1}
        onDeleteSelection={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    await user.click(getDeleteButton());
    await user.click(screen.getByRole('button', { name: 'Continuer' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('1 élément supprimé'));
  });

  it('picks the plural form for a count above one', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader
        label='pipeline'
        selectedCount={3}
        onDeleteSelection={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    await user.click(getDeleteButton());
    await user.click(screen.getByRole('button', { name: 'Continuer' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('3 éléments supprimés'));
  });

  it('translates the confirmation dialog the delete action opens', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <FeatureHeader label='pipeline' selectedCount={2} onDeleteSelection={vi.fn()} />,
    );

    await user.click(getDeleteButton());

    expect(screen.getByText('Êtes-vous absolument sûr ?')).toBeInTheDocument();
  });
});
