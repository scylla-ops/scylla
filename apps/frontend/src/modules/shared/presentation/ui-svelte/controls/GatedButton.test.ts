import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findTooltip, queryTooltip, render, textSnippet } from '@/test/render.svelte.ts';
import GatedButton from './GatedButton.svelte';

describe('GatedButton', () => {
  it('is an ordinary button when the action is allowed', async () => {
    const onclick = vi.fn();
    render(GatedButton, { children: textSnippet('New app'), onclick });

    const button = screen.getByRole('button', { name: 'New app' });
    expect(button).toBeEnabled();

    await userEvent.click(button);
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('disables itself and refuses the click when the action is denied', async () => {
    const onclick = vi.fn();
    render(GatedButton, { children: textSnippet('New app'), allowed: false, onclick });

    const button = screen.getByRole('button', { name: 'New app' });
    expect(button).toBeDisabled();

    await userEvent.click(button);
    expect(onclick).not.toHaveBeenCalled();
  });

  it('explains the denial on hover', async () => {
    render(GatedButton, {
      children: textSnippet('New app'),
      allowed: false,
      deniedReason: "You don't have permission to create apps.",
    });

    // The wrapper span, not the button: a disabled button fires no pointer
    // events, which is exactly why the span is there.
    await userEvent.hover(screen.getByRole('button', { name: 'New app' })
      .parentElement as HTMLElement);

    expect(await findTooltip()).toHaveTextContent("You don't have permission to create apps.");
  });

  it('falls back to a generic reason when the caller gives none', async () => {
    render(GatedButton, { children: textSnippet('New app'), allowed: false });

    await userEvent.hover(screen.getByRole('button', { name: 'New app' })
      .parentElement as HTMLElement);

    expect(await findTooltip()).toHaveTextContent("You don't have permission to do this.");
  });

  it('carries no tooltip at all when allowed and none was asked for', async () => {
    render(GatedButton, { children: textSnippet('New app') });

    await userEvent.hover(screen.getByRole('button', { name: 'New app' }));

    expect(queryTooltip()).toBeNull();
  });

  it('still shows a hover label when allowed, for an icon-only control', async () => {
    const onclick = vi.fn();
    render(GatedButton, { children: textSnippet('Delete'), tooltip: 'Delete', onclick });

    const button = screen.getByRole('button', { name: 'Delete' });
    // The tooltip must not cost the button its click — the denied arm disables
    // it, this one must not.
    await userEvent.click(button);
    expect(onclick).toHaveBeenCalledOnce();

    await userEvent.hover(button);
    expect(await findTooltip()).toHaveTextContent('Delete');
  });
});
