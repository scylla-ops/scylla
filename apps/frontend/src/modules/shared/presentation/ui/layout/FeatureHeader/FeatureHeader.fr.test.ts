import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { messages } from '@/modules/shared/locales/fr/messages.ts';
import { render } from '@/test/render.svelte.ts';
import { withLocale } from '@/test/i18n.ts';
import FeatureHeader from './FeatureHeader.svelte';

/** The placeholder name is part of the msgid: a renamed placeholder would lose the French plural. */
const toastSuccess = vi.fn();
vi.mock('svelte-sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

withLocale('fr', messages);

beforeEach(() => toastSuccess.mockClear());

const deleteButton = () => screen.getByRole('button', { name: 'Supprimer' });

describe('FeatureHeader in French', () => {
  it('interpolates the label into the New button', () => {
    render(FeatureHeader, { label: 'pipeline', onNew: vi.fn() });

    expect(screen.getByRole('button', { name: 'Nouveau pipeline' })).toBeInTheDocument();
  });

  it('picks the singular plural form, substituting the count for #', async () => {
    render(FeatureHeader, {
      label: 'pipeline',
      selectedCount: 1,
      onDeleteSelection: vi.fn().mockResolvedValue(undefined),
    });

    await userEvent.click(deleteButton());
    await userEvent.click(await screen.findByRole('button', { name: 'Continuer' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('1 élément supprimé'));
  });

  it('picks the plural form for a count above one', async () => {
    render(FeatureHeader, {
      label: 'pipeline',
      selectedCount: 3,
      onDeleteSelection: vi.fn().mockResolvedValue(undefined),
    });

    await userEvent.click(deleteButton());
    await userEvent.click(await screen.findByRole('button', { name: 'Continuer' }));

    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('3 éléments supprimés'));
  });

  it('translates the confirmation dialog the delete action opens', async () => {
    render(FeatureHeader, { label: 'pipeline', selectedCount: 2, onDeleteSelection: vi.fn() });

    await userEvent.click(deleteButton());

    expect(await screen.findByText('Êtes-vous absolument sûr ?')).toBeInTheDocument();
  });
});
