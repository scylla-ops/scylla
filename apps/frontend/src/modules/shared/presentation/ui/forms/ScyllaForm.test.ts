import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
// Fixtures, not the component: they are what pin the id generic. See the
// comment at the top of `DataTable.fixture.svelte`.
import ScyllaForm from './ScyllaForm.props.fixture.svelte';
import ScyllaFormFixture from './ScyllaForm.fixture.svelte';
import ScyllaFormSelectFixture from './ScyllaFormSelect.fixture.svelte';
import { items } from './form.fixture.ts';

describe('ScyllaForm', () => {
  it('renders a label and an input for each item', () => {
    render(ScyllaForm, { items, onSubmit: vi.fn(), buttonLabel: 'Save' });

    expect(screen.getByLabelText('Username')).toBeInTheDocument();
    expect(screen.getByLabelText('Bio')).toBeInTheDocument();
  });

  it('disables the submit button while a required field is empty', () => {
    render(ScyllaForm, { items, onSubmit: vi.fn(), buttonLabel: 'Save' });

    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('enables the submit button once the required field is filled', async () => {
    render(ScyllaForm, { items, onSubmit: vi.fn(), buttonLabel: 'Save' });

    await userEvent.type(screen.getByLabelText('Username'), 'ravenne');

    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
  });

  it('submits the current values, keyed by item id', async () => {
    const onSubmit = vi.fn();
    render(ScyllaForm, { items, onSubmit, buttonLabel: 'Save' });

    await userEvent.type(screen.getByLabelText('Username'), 'ravenne');
    await userEvent.type(screen.getByLabelText('Bio'), 'builds things');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith({ username: 'ravenne', bio: 'builds things' });
  });

  it('submits with an optional field left empty', async () => {
    const onSubmit = vi.fn();
    render(ScyllaForm, { items, onSubmit, buttonLabel: 'Save' });

    await userEvent.type(screen.getByLabelText('Username'), 'ravenne');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith({ username: 'ravenne', bio: '' });
  });

  it('disables every field while isPending', () => {
    render(ScyllaForm, { items, onSubmit: vi.fn(), buttonLabel: 'Save', isPending: true });

    expect(screen.getByLabelText('Username')).toBeDisabled();
    expect(screen.getByLabelText('Bio')).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('autofocuses the first item only', () => {
    render(ScyllaForm, { items, onSubmit: vi.fn(), buttonLabel: 'Save' });

    expect(screen.getByLabelText('Username')).toHaveFocus();
  });

  it('lets a footer snippet replace the default submit row, tracking isValid', async () => {
    render(ScyllaFormFixture);

    expect(screen.getByRole('button', { name: 'Incomplete' })).toBeDisabled();

    await userEvent.type(screen.getByLabelText('Username'), 'ravenne');

    expect(screen.getByRole('button', { name: 'Ready' })).toBeEnabled();
  });

  it('passes isPending through to the footer snippet', () => {
    render(ScyllaFormFixture, { isPending: true });

    expect(screen.getByRole('button', { name: 'Incomplete' })).toBeDisabled();
  });

  it('renders a select as a combobox, and submits the option chosen', async () => {
    const onSubmit = vi.fn();
    render(ScyllaFormSelectFixture, { onSubmit });

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'Pro'));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith({ plan: 'pro' });
  });
});
