import { describe, it, expect } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import ScyllaDialog from './ScyllaDialog.fixture.svelte';

const closed = () =>
  waitFor(() => {
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(document.body.style.pointerEvents).toBe('');
  });

describe('ScyllaDialog', () => {
  it('closes on its close button', async () => {
    render(ScyllaDialog);

    await userEvent.click(screen.getByRole('button', { name: 'Close' }));

    await closed();
  });

  it('closes on Escape', async () => {
    render(ScyllaDialog);

    await userEvent.keyboard('{Escape}');

    await closed();
  });

  it('closes from a button of its content', async () => {
    render(ScyllaDialog);

    await userEvent.click(screen.getByRole('button', { name: 'Done' }));

    await closed();
  });

  it('opens again after a close, with its content reset', async () => {
    render(ScyllaDialog);
    await userEvent.type(screen.getByRole('textbox', { name: 'Email' }), 'ada@example.com');
    await userEvent.keyboard('{Escape}');
    await closed();

    await userEvent.click(screen.getByRole('button', { name: 'Open' }));

    expect(await screen.findByRole('textbox', { name: 'Email' })).toHaveValue('');
  });

  it('stays open on Escape when it is not dismissible', async () => {
    render(ScyllaDialog, { dismissible: false });

    await userEvent.keyboard('{Escape}');

    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
