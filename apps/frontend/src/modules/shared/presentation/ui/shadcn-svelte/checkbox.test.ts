import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
import Checkbox from './checkbox.svelte';

describe('Checkbox', () => {
  it('exposes the checkbox role, unchecked by default', () => {
    render(Checkbox, { 'aria-label': 'Select row' });

    expect(screen.getByRole('checkbox', { name: 'Select row' })).not.toBeChecked();
  });

  it('reports its state through onCheckedChange when clicked', async () => {
    const onCheckedChange = vi.fn();
    render(Checkbox, { 'aria-label': 'Select row', onCheckedChange });

    await userEvent.click(screen.getByRole('checkbox', { name: 'Select row' }));

    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it('renders the check indicator only once checked', () => {
    const { rerender } = render(Checkbox, { 'aria-label': 'Select row', checked: false });
    // The indicator is the port's own markup: Radix mounted it conditionally,
    // here it is an `{#if}` inside the children snippet.
    expect(document.querySelector('[data-slot="checkbox-indicator"]')?.children).toHaveLength(0);

    void rerender({ 'aria-label': 'Select row', checked: true });

    expect(screen.getByRole('checkbox', { name: 'Select row' })).toBeChecked();
  });

  it('does not toggle when disabled', async () => {
    const onCheckedChange = vi.fn();
    render(Checkbox, { 'aria-label': 'Select row', disabled: true, onCheckedChange });

    await userEvent.click(screen.getByRole('checkbox', { name: 'Select row' }));

    expect(onCheckedChange).not.toHaveBeenCalled();
  });
});
