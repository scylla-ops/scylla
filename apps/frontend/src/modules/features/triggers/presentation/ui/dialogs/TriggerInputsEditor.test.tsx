import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { TriggerInputsEditor } from './TriggerInputsEditor';
import type { DraftInput } from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';

describe('TriggerInputsEditor', () => {
  it('shows a hint instead of any row when there are no inputs', () => {
    renderWithI18n(<TriggerInputsEditor inputs={[]} onChange={vi.fn()} allowJsonPointer={false} />);
    expect(screen.getByText(/injected into the run/i)).toBeInTheDocument();
    expect(screen.queryByPlaceholderText('KEY')).not.toBeInTheDocument();
  });

  it('adding an input appends a fresh literal entry', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<TriggerInputsEditor inputs={[]} onChange={onChange} allowJsonPointer={false} />);

    await user.click(screen.getByRole('button', { name: /add input/i }));

    expect(onChange).toHaveBeenCalledWith([{ key: '', valueKind: 'literal', value: '' }]);
  });

  it('editing the key of one row leaves the others untouched', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    const inputs: DraftInput[] = [
      { key: 'a', valueKind: 'literal', value: '1' },
      { key: 'b', valueKind: 'literal', value: '2' },
    ];
    renderWithI18n(<TriggerInputsEditor inputs={inputs} onChange={onChange} allowJsonPointer={false} />);

    await user.type(screen.getAllByPlaceholderText('KEY')[1], 'X');

    // The last keystroke's call: only the second row's key changed.
    const lastCall = onChange.mock.calls.at(-1)![0] as DraftInput[];
    expect(lastCall[0]).toEqual({ key: 'a', valueKind: 'literal', value: '1' });
    expect(lastCall[1].key).toBe('bX');
  });

  it('removing a row drops only that one', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    const inputs: DraftInput[] = [
      { key: 'a', valueKind: 'literal', value: '1' },
      { key: 'b', valueKind: 'literal', value: '2' },
    ];
    renderWithI18n(<TriggerInputsEditor inputs={inputs} onChange={onChange} allowJsonPointer={false} />);

    // The trash button is icon-only (no accessible name) - locate it via its
    // row, found from the row's own KEY input.
    const firstRow = screen.getAllByPlaceholderText('KEY')[0].closest('div')!;
    await user.click(firstRow.querySelector('button')!);

    expect(onChange).toHaveBeenCalledWith([{ key: 'b', valueKind: 'literal', value: '2' }]);
  });

  it('the literal/jsonPointer selector only appears when allowJsonPointer is true', () => {
    const inputs: DraftInput[] = [{ key: 'a', valueKind: 'literal', value: '1' }];
    const { rerender } = renderWithI18n(
      <TriggerInputsEditor inputs={inputs} onChange={vi.fn()} allowJsonPointer={false} />,
    );
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument();

    rerender(
      <TriggerInputsEditor inputs={inputs} onChange={vi.fn()} allowJsonPointer />,

    );
    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });

  it('the value placeholder switches based on valueKind', () => {
    const inputs: DraftInput[] = [{ key: 'a', valueKind: 'jsonPointer', value: '' }];
    renderWithI18n(<TriggerInputsEditor inputs={inputs} onChange={vi.fn()} allowJsonPointer />);
    expect(screen.getByPlaceholderText('/after')).toBeInTheDocument();
  });
});
