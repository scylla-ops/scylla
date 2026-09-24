import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findTooltip, queryTooltip, render, textSnippet } from '@/test/render.svelte.ts';
import TruncatedText from './TruncatedText.svelte';

/** jsdom lays out nothing: fake the widths. */
const setOverflow = (element: HTMLElement, { truncated }: { truncated: boolean }) => {
  Object.defineProperty(element, 'scrollWidth', {
    configurable: true,
    value: truncated ? 200 : 100,
  });
  Object.defineProperty(element, 'clientWidth', { configurable: true, value: 100 });
};

describe('TruncatedText', () => {
  it('never shows a tooltip before the text has been measured (no pointer/focus event yet)', () => {
    render(TruncatedText, { children: textSnippet('A long line of text') });

    expect(queryTooltip()).toBeNull();
  });

  it('reveals a tooltip once measured as actually overflowing', async () => {
    render(TruncatedText, { children: textSnippet('A long line of text') });

    const trigger = screen.getByRole('button');
    setOverflow(trigger, { truncated: true });
    await userEvent.hover(trigger);

    expect(await findTooltip()).toHaveTextContent('A long line of text');
  });

  it('never shows a tooltip when the text fits — repeating visible text would be noise', async () => {
    render(TruncatedText, { children: textSnippet('Short') });

    const trigger = screen.getByRole('button');
    setOverflow(trigger, { truncated: false });
    await userEvent.hover(trigger);

    expect(queryTooltip()).toBeNull();
  });

  it('shows a custom tooltip body instead of repeating the children', async () => {
    render(TruncatedText, {
      tooltip: 'Full untruncated value',
      children: textSnippet('short label'),
    });

    const trigger = screen.getByRole('button');
    setOverflow(trigger, { truncated: true });
    await userEvent.hover(trigger);

    expect(await findTooltip()).toHaveTextContent('Full untruncated value');
  });
});
