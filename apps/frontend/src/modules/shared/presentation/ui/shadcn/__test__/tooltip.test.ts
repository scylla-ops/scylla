import { describe, it, expect } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findTooltip, render } from '@/test/render.svelte.ts';
import TooltipFixture from './tooltip.fixture.svelte';

/**
 * Radix → bits-ui is the substitution with the most moving parts in this batch:
 * a provider that throws when it is missing, and an `asChild` that became a
 * snippet. Both fail at runtime, not at compile time, so they are pinned here.
 */
describe('Tooltip', () => {
  it('renders the trigger without a provider of its own — the root supplies one', () => {
    render(TooltipFixture);

    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
  });

  it('keeps the content out of the document until the trigger is hovered', () => {
    render(TooltipFixture);

    expect(screen.queryByText('Delete this secret')).toBeNull();
  });

  it('shows the content on hover, with no delay', async () => {
    render(TooltipFixture);

    await userEvent.hover(screen.getByRole('button', { name: 'Delete' }));

    await waitFor(() => expect(screen.getByText('Delete this secret')).toBeInTheDocument());
  });

  it('gives the content the tooltip role, which bits-ui leaves off', async () => {
    render(TooltipFixture);

    await userEvent.hover(screen.getByRole('button', { name: 'Delete' }));

    const tooltip = await findTooltip();
    expect(tooltip).toHaveTextContent('Delete this secret');
    // And the trigger points at it, which is what a screen reader follows.
    expect(screen.getByRole('button', { name: 'Delete' })).toHaveAttribute(
      'aria-describedby',
      tooltip.id,
    );
  });

  it('shows the content on keyboard focus, which is the accessible path', async () => {
    render(TooltipFixture);

    screen.getByRole('button', { name: 'Delete' }).focus();

    await waitFor(() => expect(screen.getByText('Delete this secret')).toBeInTheDocument());
  });

  it('lets the trigger be the caller own button through the child snippet', async () => {
    render(TooltipFixture, { asChild: true });

    // One button, not a button inside a button: `child` hands its props over
    // instead of rendering a trigger of its own, which is what `asChild` did.
    const trigger = screen.getByRole('button', { name: 'Delete' });
    expect(screen.getAllByRole('button')).toHaveLength(1);
    // It is our Button — `data-variant` is the half of its identity the merge
    // cannot overwrite — wearing the trigger's props on top.
    expect(trigger).toHaveAttribute('data-variant', 'default');
    expect(trigger).toHaveAttribute('data-slot', 'tooltip-trigger');

    await userEvent.hover(trigger);

    await waitFor(() => expect(screen.getByText('Delete this secret')).toBeInTheDocument());
  });
});
