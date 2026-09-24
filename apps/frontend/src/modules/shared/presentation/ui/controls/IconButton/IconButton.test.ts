import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import PencilIcon from '@lucide/svelte/icons/pencil';
import { findTooltip, render } from '@/test/render.svelte.ts';
import IconButton from './IconButton.svelte';

describe('IconButton', () => {
  it('is findable by the tooltip text, which is its accessible name', () => {
    render(IconButton, { icon: PencilIcon, tooltip: 'Edit secret' });

    // The visually-hidden label, not the tooltip: a tooltip is only wired up as
    // `aria-describedby`, so while it is closed the button would have no name.
    expect(screen.getByRole('button', { name: 'Edit secret' })).toBeInTheDocument();
  });

  it('opens its tooltip on hover', async () => {
    render(IconButton, { icon: PencilIcon, tooltip: 'Edit secret' });

    await userEvent.hover(screen.getByRole('button', { name: 'Edit secret' }));

    expect(await findTooltip()).toHaveTextContent('Edit secret');
  });

  it('fires its click handler', async () => {
    const onclick = vi.fn();
    render(IconButton, { icon: PencilIcon, tooltip: 'Edit secret', onclick });

    await userEvent.click(screen.getByRole('button', { name: 'Edit secret' }));

    expect(onclick).toHaveBeenCalledOnce();
  });

  it('does not fire when disabled', async () => {
    const onclick = vi.fn();
    render(IconButton, { icon: PencilIcon, tooltip: 'Edit secret', disabled: true, onclick });

    await userEvent.click(screen.getByRole('button', { name: 'Edit secret' }));

    expect(onclick).not.toHaveBeenCalled();
  });

  it('marks itself busy and stops responding while the action runs', async () => {
    const onclick = vi.fn();
    render(IconButton, { icon: PencilIcon, tooltip: 'Edit secret', busy: true, onclick });

    const button = screen.getByRole('button', { name: 'Edit secret' });
    expect(button).toHaveAttribute('aria-busy', 'true');

    await userEvent.click(button);

    // `busy` is one prop, not a `disabled` and a spinner that can drift apart.
    expect(onclick).not.toHaveBeenCalled();
  });
});
