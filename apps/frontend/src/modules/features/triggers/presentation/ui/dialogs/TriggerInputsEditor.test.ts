import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findFloating, render } from '@/test/render.svelte.ts';
import type { DraftInput } from '../../utils/trigger-form.utils.ts';
import TriggerInputsEditor from './TriggerInputsEditor.svelte';

const literal = (overrides: Partial<DraftInput> = {}): DraftInput => ({
  key: 'BRANCH',
  valueKind: 'literal',
  value: 'main',
  ...overrides,
});

describe('TriggerInputsEditor', () => {
  it('explains what inputs are for when there are none', () => {
    render(TriggerInputsEditor, { inputs: [], onChange: vi.fn(), allowJsonPointer: false });

    expect(
      screen.getByText('Optional values injected into the run as environment variables.'),
    ).toBeInTheDocument();
  });

  it('appends a blank row when "Add input" is pressed', async () => {
    const onChange = vi.fn();
    render(TriggerInputsEditor, { inputs: [], onChange, allowJsonPointer: false });

    await userEvent.click(screen.getByRole('button', { name: 'Add input' }));

    expect(onChange).toHaveBeenCalledWith([{ key: '', valueKind: 'literal', value: '' }]);
  });

  it('keeps the existing rows when appending', async () => {
    const onChange = vi.fn();
    render(TriggerInputsEditor, { inputs: [literal()], onChange, allowJsonPointer: false });

    await userEvent.click(screen.getByRole('button', { name: 'Add input' }));

    expect(onChange).toHaveBeenCalledWith([literal(), { key: '', valueKind: 'literal', value: '' }]);
  });

  it('edits only the row that was typed in', async () => {
    const onChange = vi.fn();
    render(TriggerInputsEditor, {
      inputs: [literal({ key: 'A' }), literal({ key: 'B' })],
      onChange,
      allowJsonPointer: false,
    });

    const [firstKeyField] = screen.getAllByPlaceholderText('KEY');
    await userEvent.type(firstKeyField, 'X');

    // One keystroke on a controlled field: the patch carries the whole list
    // back, with only the first row changed.
    expect(onChange).toHaveBeenLastCalledWith([literal({ key: 'AX' }), literal({ key: 'B' })]);
  });

  it('removes the row whose delete button was pressed, not the first one', async () => {
    const onChange = vi.fn();
    render(TriggerInputsEditor, {
      inputs: [literal({ key: 'A' }), literal({ key: 'B' })],
      onChange,
      allowJsonPointer: false,
    });

    const [, secondDelete] = screen.getAllByRole('button', { name: 'Remove input' });
    await userEvent.click(secondDelete);

    expect(onChange).toHaveBeenCalledWith([literal({ key: 'A' })]);
  });

  it('hides the value-kind selector for a cron trigger, which has no payload to point into', () => {
    render(TriggerInputsEditor, {
      inputs: [literal()],
      onChange: vi.fn(),
      allowJsonPointer: false,
    });

    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  });

  it('offers the JSON-pointer kind for a webhook trigger', async () => {
    const onChange = vi.fn();
    render(TriggerInputsEditor, { inputs: [literal()], onChange, allowJsonPointer: true });

    await userEvent.click(screen.getByRole('combobox'));
    await userEvent.click(await findFloating('option', 'JSON pointer'));

    expect(onChange).toHaveBeenCalledWith([literal({ valueKind: 'jsonPointer' })]);
  });

  it('hints at a pointer path once a row is a JSON pointer', () => {
    render(TriggerInputsEditor, {
      inputs: [literal({ valueKind: 'jsonPointer', value: '/after' })],
      onChange: vi.fn(),
      allowJsonPointer: true,
    });

    expect(screen.getByPlaceholderText('/after')).toBeInTheDocument();
    expect(screen.queryByPlaceholderText('value')).not.toBeInTheDocument();
  });
});
