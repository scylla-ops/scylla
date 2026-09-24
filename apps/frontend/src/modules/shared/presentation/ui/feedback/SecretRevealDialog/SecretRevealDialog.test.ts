import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { render, textSnippet } from '@/test/render.svelte.ts';
import SecretRevealDialog from './SecretRevealDialog.svelte';

const props = {
  open: true,
  title: 'Agent created',
  description: 'Copy the secret now.',
  secret: 'sk-super-secret',
  secretLabel: 'agent-1',
  onClose: vi.fn(),
};

const doneButton = () => screen.getByRole('button', { name: /done/i });
const revealButton = () => screen.getByRole('button', { name: /reveal/i });

describe('SecretRevealDialog', () => {
  it('starts with the secret blurred and Done disabled', () => {
    render(SecretRevealDialog, { ...props, onClose: vi.fn() });

    expect(doneButton()).toBeDisabled();
    expect(revealButton()).toBeInTheDocument();
  });

  it('reveals the secret and enables Done once Reveal is clicked', async () => {
    render(SecretRevealDialog, { ...props, onClose: vi.fn() });

    await userEvent.click(revealButton());

    expect(doneButton()).toBeEnabled();
    expect(screen.queryByRole('button', { name: /reveal/i })).not.toBeInTheDocument();
  });

  it('calls onClose when Done is clicked, only reachable after revealing', async () => {
    const onClose = vi.fn();
    render(SecretRevealDialog, { ...props, onClose });

    await userEvent.click(revealButton());
    await userEvent.click(doneButton());

    expect(onClose).toHaveBeenCalled();
  });

  it('resets to the hidden phase when closed and reopened, rather than staying revealed', async () => {
    const { rerender } = render(SecretRevealDialog, { ...props, onClose: vi.fn() });

    await userEvent.click(revealButton());
    expect(doneButton()).toBeEnabled();

    await rerender({ open: false });
    await rerender({ open: true });

    expect(doneButton()).toBeDisabled();
    expect(revealButton()).toBeInTheDocument();
  });

  it('shows the second step, dimmed, only revealed once the secret is', async () => {
    render(SecretRevealDialog, {
      ...props,
      onClose: vi.fn(),
      secondStep: { title: 'Start the worker', content: textSnippet('docker run ...') },
    });

    expect(screen.getByText('Start the worker')).toBeInTheDocument();
    expect(screen.queryByText('docker run ...')).not.toBeInTheDocument();

    await userEvent.click(revealButton());

    expect(screen.getByText('docker run ...')).toBeInTheDocument();
  });

  it('shows a revealedNote instead, when there is no secondStep', async () => {
    render(SecretRevealDialog, {
      ...props,
      onClose: vi.fn(),
      revealedNote: 'Keep it somewhere safe.',
    });

    expect(screen.queryByText('Keep it somewhere safe.')).not.toBeInTheDocument();

    await userEvent.click(revealButton());

    expect(screen.getByText('Keep it somewhere safe.')).toBeInTheDocument();
  });

  it('shows the default footer note, or a custom one', () => {
    render(SecretRevealDialog, {
      ...props,
      onClose: vi.fn(),
      footerNote: 'This secret only works for 24h.',
    });

    expect(screen.getByText('This secret only works for 24h.')).toBeInTheDocument();
  });
});
