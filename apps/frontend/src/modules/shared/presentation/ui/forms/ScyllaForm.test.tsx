import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ScyllaForm } from './ScyllaForm';
import { FormItemType, type FormItem } from '@shared/presentation/structs/scylla-form.struct.ts';

type Ids = 'name' | 'description';

const items: readonly FormItem<Ids>[] = [
  { id: 'name', type: FormItemType.Input, inputType: 'text', label: 'Name' },
  {
    id: 'description',
    type: FormItemType.Input,
    inputType: 'text',
    label: 'Description',
    optional: true,
  },
];

describe('ScyllaForm', () => {
  it('renders a label and input for each item', () => {
    render(<ScyllaForm items={items} onSubmit={vi.fn()} />);
    expect(screen.getByLabelText('Name')).toBeInTheDocument();
    expect(screen.getByLabelText('Description')).toBeInTheDocument();
  });

  it('the submit button is disabled while a required field is empty', () => {
    render(<ScyllaForm items={items} onSubmit={vi.fn()} buttonLabel='Save' />);
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('typing into a required field enables the submit button', async () => {
    const user = userEvent.setup();
    render(<ScyllaForm items={items} onSubmit={vi.fn()} buttonLabel='Save' />);

    await user.type(screen.getByLabelText('Name'), 'my-agent');

    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
  });

  it('submits the current values on submit click', async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();
    render(<ScyllaForm items={items} onSubmit={onSubmit} buttonLabel='Save' />);

    await user.type(screen.getByLabelText('Name'), 'my-agent');
    await user.type(screen.getByLabelText('Description'), 'a test agent');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith({ name: 'my-agent', description: 'a test agent' });
  });

  it('submits with the optional field left empty', async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();
    render(<ScyllaForm items={items} onSubmit={onSubmit} buttonLabel='Save' />);

    await user.type(screen.getByLabelText('Name'), 'my-agent');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith({ name: 'my-agent', description: '' });
  });

  it('disables every field while isPending', () => {
    render(<ScyllaForm items={items} onSubmit={vi.fn()} isPending buttonLabel='Save' />);
    expect(screen.getByLabelText('Name')).toBeDisabled();
    expect(screen.getByLabelText('Description')).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('autofocuses the first item only', () => {
    render(<ScyllaForm items={items} onSubmit={vi.fn()} />);
    expect(screen.getByLabelText('Name')).toHaveFocus();
    expect(screen.getByLabelText('Description')).not.toHaveFocus();
  });

  it('a custom footer render prop replaces the default submit button, receiving isValid/isPending', () => {
    render(
      <ScyllaForm
        items={items}
        onSubmit={vi.fn()}
        footer={({ isValid, isPending }) => (
          <p>{`valid:${String(isValid)} pending:${String(isPending)}`}</p>
        )}
      />,
    );
    expect(screen.getByText('valid:false pending:false')).toBeInTheDocument();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});
