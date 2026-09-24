import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, withQueryClient } from '@/test/render.svelte.ts';
import StepNodeForm from './StepNodeForm.svelte';

let restore: () => void;
const onSubmit = vi.fn();
const onCancel = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  restore = withQueryClient().restore;
});

afterEach(() => restore());

describe('StepNodeForm', () => {
  it('does not submit a script step with an empty script', async () => {
    const user = userEvent.setup();
    render(StepNodeForm, { onSubmit, onCancel });

    await user.type(screen.getByLabelText('Node ID'), 'build');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('does not submit a step without an id', async () => {
    const user = userEvent.setup();
    render(StepNodeForm, { onSubmit, onCancel });

    await user.click(screen.getByRole('button', { name: 'Command' }));
    await user.type(screen.getByLabelText('Command'), 'cargo');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('submits a command step with its arguments and variables', async () => {
    const user = userEvent.setup();
    render(StepNodeForm, { onSubmit, onCancel });

    await user.type(screen.getByLabelText('Node ID'), 'build');
    await user.click(screen.getByRole('button', { name: 'Command' }));
    await user.type(screen.getByLabelText('Command'), 'cargo');
    await user.click(screen.getByRole('button', { name: 'Add argument' }));
    await user.type(screen.getByPlaceholderText('e.g., --release'), 'build');
    await user.click(screen.getByRole('button', { name: 'Add variable' }));
    await user.type(screen.getByPlaceholderText('KEY'), 'MODE');
    await user.type(screen.getByPlaceholderText('value'), 'ci');
    await user.type(screen.getByLabelText('Working directory'), 'crates');
    await user.click(screen.getByRole('button', { name: 'Add Node' }));

    expect(onSubmit).toHaveBeenCalledWith('build', {
      kind: 'exec',
      command: 'cargo',
      args: ['build'],
      workingDir: 'crates',
      env: [{ key: 'MODE', kind: 'literal', value: 'ci' }],
    });
  });

  it('removes an argument and a variable', async () => {
    const user = userEvent.setup();
    render(StepNodeForm, { onSubmit, onCancel });

    await user.click(screen.getByRole('button', { name: 'Command' }));
    await user.click(screen.getByRole('button', { name: 'Add argument' }));
    await user.click(screen.getByRole('button', { name: 'Remove argument' }));
    await user.click(screen.getByRole('button', { name: 'Add variable' }));
    await user.click(screen.getByRole('button', { name: 'Remove variable' }));

    expect(screen.queryByPlaceholderText('e.g., --release')).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText('KEY')).not.toBeInTheDocument();
  });

  it('edits an existing step and saves it', async () => {
    const user = userEvent.setup();
    render(StepNodeForm, {
      onSubmit,
      onCancel,
      editingStep: {
        id: 'build',
        deps: [],
        kind: 'script',
        script: 'make',
        shell: 'bash',
        env: [{ key: 'TOKEN', kind: 'secret', secretRef: 'GH' }],
      },
    });

    expect(screen.getByLabelText('Node ID')).toHaveValue('build');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(onSubmit).toHaveBeenCalledWith('build', {
      kind: 'script',
      script: 'make',
      shell: 'bash',
      workingDir: undefined,
      env: [{ key: 'TOKEN', kind: 'secret', secretRef: 'GH' }],
    });
  });

  it('cancels', async () => {
    render(StepNodeForm, { onSubmit, onCancel });

    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onCancel).toHaveBeenCalled();
  });
});
