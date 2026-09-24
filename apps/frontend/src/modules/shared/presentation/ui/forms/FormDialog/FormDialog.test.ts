import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/render.svelte.ts';
// The fixture, not the component: it is what pins the id generic. See the
// comment at the top of `DataTable.fixture.svelte`.
import FormDialog from './FormDialog.fixture.svelte';
import { FormItemType, type FormItem } from '../scylla-form.struct.ts';

const items: readonly FormItem<'name'>[] = [
  { id: 'name', type: FormItemType.Input, inputType: 'text', label: 'Name' },
];

const base = { items, onOpenChange: vi.fn(), onSubmit: vi.fn(), title: 'New secret' };

describe('FormDialog', () => {
  it('renders nothing while closed', () => {
    render(FormDialog, { ...base, open: false });

    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('is titled and described by its own header', async () => {
    render(FormDialog, { ...base, open: true, description: 'Available to every pipeline.' });

    const dialog = await screen.findByRole('dialog');
    expect(dialog).toHaveAccessibleName('New secret');
    expect(dialog).toHaveAccessibleDescription('Available to every pipeline.');
  });

  it('keeps Create disabled until the form is valid, then submits the values', async () => {
    const onSubmit = vi.fn();
    render(FormDialog, { ...base, open: true, onSubmit });

    await screen.findByRole('dialog');
    expect(screen.getByRole('button', { name: 'Create' })).toBeDisabled();

    await userEvent.type(screen.getByLabelText('Name'), 'api-token');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));

    expect(onSubmit).toHaveBeenCalledWith({ name: 'api-token' });
  });

  it('closes through its Cancel button without submitting', async () => {
    const onOpenChange = vi.fn();
    const onSubmit = vi.fn();
    render(FormDialog, { ...base, open: true, onOpenChange, onSubmit });

    await userEvent.click(await screen.findByRole('button', { name: 'Cancel' }));

    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('shows the pending label and disables both buttons while the mutation runs', async () => {
    render(FormDialog, { ...base, open: true, isPending: true });

    expect(await screen.findByRole('button', { name: 'Creating...' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
  });

  it('uses caller-supplied submit and pending labels', async () => {
    const { rerender } = render(FormDialog, {
      ...base,
      open: true,
      submitLabel: 'Save',
      pendingLabel: 'Saving...',
    });
    expect(await screen.findByRole('button', { name: 'Save' })).toBeInTheDocument();

    await rerender({
      ...base,
      open: true,
      submitLabel: 'Save',
      pendingLabel: 'Saving...',
      isPending: true,
    });

    expect(screen.getByRole('button', { name: 'Saving...' })).toBeInTheDocument();
  });

  it('leaves only one way out when hideCancel is set', async () => {
    render(FormDialog, { ...base, open: true, hideCancel: true });

    await screen.findByRole('dialog');

    expect(screen.queryByRole('button', { name: 'Cancel' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Create' })).toBeInTheDocument();
  });
});
