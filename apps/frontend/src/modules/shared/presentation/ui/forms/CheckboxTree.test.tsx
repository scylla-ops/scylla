import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CheckboxTree } from './CheckboxTree';
import type { CheckboxNode } from './CheckboxTree';

// Radix Collapsible measures its content's height for the open/close
// animation, which needs ResizeObserver - absent in jsdom.
class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const tree: CheckboxNode[] = [
  {
    id: 'read',
    label: 'Read',
    children: [
      { id: 'update', label: 'Update' },
      { id: 'delete', label: 'Delete' },
    ],
  },
  { id: 'list', label: 'List' },
];

describe('CheckboxTree', () => {
  it('renders a label for every node, nested included', () => {
    render(<CheckboxTree nodes={tree} />);
    expect(screen.getByLabelText('Read')).toBeInTheDocument();
    expect(screen.getByLabelText('Update')).toBeInTheDocument();
    expect(screen.getByLabelText('Delete')).toBeInTheDocument();
    expect(screen.getByLabelText('List')).toBeInTheDocument();
  });

  it('checking a leaf calls onCheckedChange with it included', async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(<CheckboxTree nodes={tree} onCheckedChange={onCheckedChange} />);

    await user.click(screen.getByLabelText('List'));

    expect(onCheckedChange).toHaveBeenCalledWith(['list']);
  });

  it('checking a parent does not implicitly check its children', async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(<CheckboxTree nodes={tree} onCheckedChange={onCheckedChange} />);

    await user.click(screen.getByLabelText('Read'));

    expect(onCheckedChange).toHaveBeenCalledWith(['read']);
    expect(screen.getByLabelText('Update')).not.toBeChecked();
  });

  it('unchecking a parent cascades to uncheck every descendant', async () => {
    const onCheckedChange = vi.fn();
    const user = userEvent.setup();
    render(
      <CheckboxTree nodes={tree} defaultCheckedIds={['read', 'update', 'delete']} onCheckedChange={onCheckedChange} />,
    );

    expect(screen.getByLabelText('Update')).toBeChecked();
    await user.click(screen.getByLabelText('Read'));

    expect(screen.getByLabelText('Update')).not.toBeChecked();
    expect(screen.getByLabelText('Delete')).not.toBeChecked();
    expect(onCheckedChange).toHaveBeenCalledWith([]);
  });

  it('a child seeded checked without its parent being checked is dropped entirely (parent chain not satisfied)', () => {
    render(<CheckboxTree nodes={tree} defaultCheckedIds={['update']} />);
    expect(screen.getByLabelText('Update')).not.toBeChecked();
  });

  it('a child is checked once both it and its parent are seeded checked', () => {
    render(<CheckboxTree nodes={tree} defaultCheckedIds={['read', 'update']} />);
    expect(screen.getByLabelText('Read')).toBeChecked();
    expect(screen.getByLabelText('Update')).toBeChecked();
    expect(screen.getByLabelText('Delete')).not.toBeChecked();
  });

  it('a child\'s checkbox is disabled while its parent is unchecked', () => {
    render(<CheckboxTree nodes={tree} />);
    expect(screen.getByLabelText('Update')).toBeDisabled();
  });

  it('a child\'s checkbox becomes enabled once its parent is checked', async () => {
    const user = userEvent.setup();
    render(<CheckboxTree nodes={tree} />);

    await user.click(screen.getByLabelText('Read'));

    expect(screen.getByLabelText('Update')).toBeEnabled();
  });

  it('allDisabled disables every checkbox without altering the current selection', () => {
    render(<CheckboxTree nodes={tree} defaultCheckedIds={['read', 'update']} allDisabled />);
    expect(screen.getByLabelText('Read')).toBeDisabled();
    expect(screen.getByLabelText('Read')).toBeChecked();
    expect(screen.getByLabelText('List')).toBeDisabled();
  });

  it('re-seeds when defaultCheckedIds actually changes', () => {
    const { rerender } = render(<CheckboxTree nodes={tree} defaultCheckedIds={['list']} />);
    expect(screen.getByLabelText('List')).toBeChecked();

    rerender(<CheckboxTree nodes={tree} defaultCheckedIds={['read']} />);

    expect(screen.getByLabelText('List')).not.toBeChecked();
    expect(screen.getByLabelText('Read')).toBeChecked();
  });

  it('does not reset an in-progress selection when the parent merely echoes back what was just emitted', async () => {
    const user = userEvent.setup();
    let lastEmitted: string[] = [];
    const onCheckedChange = vi.fn((ids: string[]) => {
      lastEmitted = ids;
    });

    const { rerender } = render(<CheckboxTree nodes={tree} onCheckedChange={onCheckedChange} />);
    await user.click(screen.getByLabelText('Read'));
    expect(lastEmitted).toEqual(['read']);

    // The parent feeds the emitted selection straight back as defaultCheckedIds
    // (the common controlled-component pattern) - this must not clobber
    // whatever the tree already has selected.
    rerender(<CheckboxTree nodes={tree} defaultCheckedIds={lastEmitted} onCheckedChange={onCheckedChange} />);

    expect(screen.getByLabelText('Read')).toBeChecked();
  });
});
