import { describe, it, expect } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
import DialogFixture from './dialog.fixture.svelte';

describe('Dialog', () => {
  it('renders nothing until the trigger is used', () => {
    render(DialogFixture);

    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('opens on the trigger, titled and described', async () => {
    render(DialogFixture);

    await userEvent.click(screen.getByRole('button', { name: 'Open' }));

    const dialog = await screen.findByRole('dialog');
    // Both come from the primitive's own `aria-labelledby`/`aria-describedby`
    // wiring: the title and description must be bits-ui's, not plain divs.
    expect(dialog).toHaveAccessibleName('Create a secret');
    expect(dialog).toHaveAccessibleDescription('It will be available to every pipeline.');
  });

  it('gives its close button an accessible name, which the icon alone does not', async () => {
    render(DialogFixture);

    await userEvent.click(screen.getByRole('button', { name: 'Open' }));
    await screen.findByRole('dialog');

    expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
  });

  it('closes on its close button', async () => {
    render(DialogFixture);

    await userEvent.click(screen.getByRole('button', { name: 'Open' }));
    await screen.findByRole('dialog');

    await userEvent.click(screen.getByRole('button', { name: 'Close' }));

    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('closes on Escape', async () => {
    render(DialogFixture);

    await userEvent.click(screen.getByRole('button', { name: 'Open' }));
    await screen.findByRole('dialog');

    await userEvent.keyboard('{Escape}');

    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });
});
