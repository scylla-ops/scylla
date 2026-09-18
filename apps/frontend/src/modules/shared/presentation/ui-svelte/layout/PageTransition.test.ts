import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render, textSnippet } from '@/test/render.svelte.ts';
import PageTransition from './PageTransition.svelte';

describe('PageTransition', () => {
  it('renders its children in a main landmark', () => {
    render(PageTransition, { key: '/secrets', children: textSnippet('Secrets page') });

    expect(screen.getByRole('main')).toHaveTextContent('Secrets page');
  });

  it('keeps both pages mounted across a key change, which is what React could not do', async () => {
    const { rerender } = render(PageTransition, {
      key: '/secrets',
      children: textSnippet('Secrets page'),
    });

    await rerender({ key: '/jobs', children: textSnippet('Jobs page') });

    // The outgoing node is still there, mid `out:` transition — that is the
    // leave animation the Lot A cleanup had to give up.
    expect(screen.getAllByRole('main')).toHaveLength(2);
  });

  it('stacks the two panes so the departing one cannot push the arriving one down', () => {
    render(PageTransition, { key: '/secrets', children: textSnippet('Secrets page') });

    expect(screen.getByRole('main')).toHaveClass('absolute', 'inset-0');
  });

  it('replays nothing while the key is unchanged', async () => {
    const { rerender } = render(PageTransition, {
      key: '/secrets',
      children: textSnippet('Secrets page'),
    });

    await rerender({ key: '/secrets', children: textSnippet('Secrets page, refreshed') });

    expect(screen.getAllByRole('main')).toHaveLength(1);
  });
});
