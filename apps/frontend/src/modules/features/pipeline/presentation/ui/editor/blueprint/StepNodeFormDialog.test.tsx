import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { StepNodeFormDialog } from './StepNodeFormDialog';
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';
import type * as ReactCodeMirrorModule from '@uiw/react-codemirror';

const paramsMock = vi.fn(() => ({ projectId: 'project-1' }));
vi.mock('react-router-dom', () => ({
  useParams: () => paramsMock(),
}));

const secretsMock = vi.fn(() => ({ secrets: [] as { id: string; name: string }[] }));
vi.mock('@/modules/features/secret', () => ({
  useSecrets: () => secretsMock(),
}));

// The real editor needs a lot jsdom can't give it (measurement, ResizeObserver,
// ProseMirror-style DOM ranges) - a plain textarea captures the same
// value/onChange contract this dialog actually depends on. Only the default
// export (the component) is replaced - useCodeMirrorTheme separately imports
// EditorView from this same package, which must stay real.
vi.mock('@uiw/react-codemirror', async importOriginal => {
  const actual = await importOriginal<typeof ReactCodeMirrorModule>();
  return {
    ...actual,
    default: ({
      value,
      onChange,
      placeholder,
    }: {
      value: string;
      onChange: (v: string) => void;
      placeholder?: string;
    }) => (
      <textarea
        data-testid='script-editor'
        value={value}
        placeholder={placeholder}
        onChange={e => onChange(e.target.value)}
      />
    ),
  };
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

beforeEach(() => {
  paramsMock.mockReturnValue({ projectId: 'project-1' });
  secretsMock.mockReturnValue({ secrets: [] });
  Element.prototype.hasPointerCapture = vi.fn().mockReturnValue(false);
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

describe('StepNodeFormDialog', () => {
  it('add mode: titles the dialog "Add a new node" and defaults to script mode', () => {
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={vi.fn()} onEdit={vi.fn()} />);
    expect(screen.getByText('Add a new node')).toBeInTheDocument();
    expect(screen.getByTestId('script-editor')).toBeInTheDocument();
    expect(screen.queryByLabelText('Command')).not.toBeInTheDocument();
  });

  it('switching to command mode swaps the script editor for a command + arguments UI', async () => {
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={vi.fn()} onEdit={vi.fn()} />);

    await user.click(screen.getByText('Command'));
    expect(screen.queryByTestId('script-editor')).not.toBeInTheDocument();
    expect(screen.getByLabelText('Command')).toBeInTheDocument();
    expect(screen.getByText('Add argument')).toBeInTheDocument();
  });

  it('submitting a script node calls onAdd with the trimmed id and script payload', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), '  build  ');
    await user.type(screen.getByTestId('script-editor'), 'cargo build');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onAdd).toHaveBeenCalledWith('build', {
      kind: 'script',
      script: 'cargo build',
      shell: 'sh',
      workingDir: undefined,
      env: [],
    });
  });

  it('an empty script blocks submission entirely', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), 'build');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));
    expect(onAdd).not.toHaveBeenCalled();
  });

  it('an empty node id blocks submission entirely', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByTestId('script-editor'), 'cargo build');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));
    expect(onAdd).not.toHaveBeenCalled();
  });

  it('command mode: adds/edits/removes argument rows and only keeps non-empty ones on submit', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), 'test');
    await user.click(screen.getByText('Command'));
    await user.type(screen.getByLabelText('Command'), 'cargo test');

    await user.click(screen.getByText('Add argument'));
    await user.click(screen.getByText('Add argument'));
    const argInputs = screen.getAllByPlaceholderText('e.g., --release');
    await user.type(argInputs[0], '--release');
    // Second row added but left blank, then removed via its trash button.
    const secondRow = argInputs[1].closest('div')!;
    await user.click(secondRow.querySelector('button')!);

    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onAdd).toHaveBeenCalledWith('test', {
      kind: 'exec',
      command: 'cargo test',
      args: ['--release'],
      workingDir: undefined,
      env: [],
    });
  });

  it('a blank command in command mode blocks submission', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), 'test');
    await user.click(screen.getByText('Command'));
    await user.click(screen.getByRole('button', { name: 'Add Node' }));
    expect(onAdd).not.toHaveBeenCalled();
  });

  it('a literal env row with a blank key is dropped, a filled one is trimmed and included', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), 'build');
    await user.type(screen.getByTestId('script-editor'), 'go build');

    await user.click(screen.getByText('Add variable'));
    await user.click(screen.getByText('Add variable'));
    const keyInputs = screen.getAllByPlaceholderText('KEY');
    await user.type(keyInputs[0], '  TOKEN  ');
    await user.type(screen.getAllByPlaceholderText('value')[0], 'abc123');
    // keyInputs[1] stays blank - dropped from the payload.

    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onAdd).toHaveBeenCalledWith('build', {
      kind: 'script',
      script: 'go build',
      shell: 'sh',
      workingDir: undefined,
      env: [{ key: 'TOKEN', kind: 'literal', value: 'abc123' }],
    });
  });

  it('a working directory is trimmed, and left as undefined when blank', async () => {
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={vi.fn()} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.type(screen.getByLabelText('Node ID'), 'build');
    await user.type(screen.getByTestId('script-editor'), 'go build');
    await user.type(screen.getByLabelText('Working directory'), '  ./services/api  ');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onAdd).toHaveBeenCalledWith(
      'build',
      expect.objectContaining({ workingDir: './services/api' }),
    );
  });

  it('edit mode: prefills every field from the editing node and titles the dialog "Edit node"', () => {
    const editingNode: PipelineNodeData = {
      id: 'deploy',
      deps: [],
      kind: 'exec',
      command: 'kubectl apply',
      args: ['-f', 'manifest.yaml'],
      workingDir: '/infra',
      env: [{ key: 'ENV', kind: 'literal', value: 'prod' }],
    };
    renderWithI18n(
      <StepNodeFormDialog open onOpenChange={vi.fn()} editingNode={editingNode} onAdd={vi.fn()} onEdit={vi.fn()} />,
    );

    expect(screen.getByText('Edit node')).toBeInTheDocument();
    expect(screen.getByLabelText('Node ID')).toHaveValue('deploy');
    expect(screen.getByLabelText('Command')).toHaveValue('kubectl apply');
    expect(screen.getByDisplayValue('-f')).toBeInTheDocument();
    expect(screen.getByDisplayValue('manifest.yaml')).toBeInTheDocument();
    expect(screen.getByLabelText('Working directory')).toHaveValue('/infra');
    expect(screen.getByPlaceholderText('KEY')).toHaveValue('ENV');
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
  });

  it('editing a node calls onEdit with the original id, the new id and the updated payload', async () => {
    const onEdit = vi.fn();
    const user = userEvent.setup();
    const editingNode: PipelineNodeData = {
      id: 'build',
      deps: [],
      kind: 'script',
      script: 'cargo build',
      shell: 'sh',
      env: [],
    };
    renderWithI18n(
      <StepNodeFormDialog open onOpenChange={vi.fn()} editingNode={editingNode} onAdd={vi.fn()} onEdit={onEdit} />,
    );

    const idInput = screen.getByLabelText('Node ID');
    await user.clear(idInput);
    await user.type(idInput, 'build-renamed');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(onEdit).toHaveBeenCalledWith('build', 'build-renamed', {
      kind: 'script',
      script: 'cargo build',
      shell: 'sh',
      workingDir: undefined,
      env: [],
    });
  });

  it('Cancel closes the dialog without submitting', async () => {
    const onOpenChange = vi.fn();
    const onAdd = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(<StepNodeFormDialog open onOpenChange={onOpenChange} onAdd={onAdd} onEdit={vi.fn()} />);

    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(onAdd).not.toHaveBeenCalled();
  });
});
