import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { messages } from '@/modules/shared/locales/fr/messages.ts';
import { render } from '@/test/render.svelte.ts';
import { withLocale } from '@/test/i18n.ts';
import FeatureHeader from './FeatureHeader.svelte';

/**
 * The Svelte side's one real-translation test, and it exists for a specific
 * reason beyond mirroring `FeatureHeader.fr.test.tsx`.
 *
 * Messages moved out of the component into `feature-header.messages.ts`, and a
 * ported message only keeps its translation if its **msgid** is byte-identical —
 * which includes the placeholder names. `{count, plural, …}` and
 * `{selectedCount, plural, …}` are two different messages. Nothing would have
 * failed had they diverged: `pnpm extract` would have written a second, empty
 * entry, `i18n:collisions` would still report zero, and this arm would render
 * in English. This test is the thing that notices.
 */
const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
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
