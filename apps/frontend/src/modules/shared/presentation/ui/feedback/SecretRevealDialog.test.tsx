import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { SecretRevealDialog } from './SecretRevealDialog';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

describe('SecretRevealDialog', () => {
  it('starts with the secret blurred and Done disabled', () => {
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByRole('button', { name: /done/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /reveal/i })).toBeInTheDocument();
  });

  it('reveals the secret and enables Done once Reveal is clicked', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        onClose={vi.fn()}
      />,
    );

    await user.click(screen.getByRole('button', { name: /reveal/i }));

    expect(screen.getByRole('button', { name: /done/i })).toBeEnabled();
    expect(screen.queryByRole('button', { name: /reveal/i })).not.toBeInTheDocument();
  });

  it('calls onClose when Done is clicked, only reachable after revealing', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        onClose={onClose}
      />,
    );

    await user.click(screen.getByRole('button', { name: /reveal/i }));
    await user.click(screen.getByRole('button', { name: /done/i }));

    expect(onClose).toHaveBeenCalled();
  });

  it('resets to the hidden phase when closed and reopened, rather than staying revealed', async () => {
    const user = userEvent.setup();
    const props = {
      title: 'Agent created',
      description: 'Copy the secret now.',
      secret: 'sk-super-secret',
      secretLabel: 'agent-1',
      onClose: vi.fn(),
    };
    const { rerender } = renderWithI18n(<SecretRevealDialog open {...props} />);

    await user.click(screen.getByRole('button', { name: /reveal/i }));
    expect(screen.getByRole('button', { name: /done/i })).toBeEnabled();

    rerender(
      <I18nProvider i18n={i18n}>
        <SecretRevealDialog open={false} {...props} />
      </I18nProvider>,
    );
    rerender(
      <I18nProvider i18n={i18n}>
        <SecretRevealDialog open {...props} />
      </I18nProvider>,
    );

    expect(screen.getByRole('button', { name: /done/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /reveal/i })).toBeInTheDocument();
  });

  it('shows the second step, dimmed, only revealed once the secret is', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        secondStep={{ title: 'Start the worker', content: <span>docker run ...</span> }}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByText('Start the worker')).toBeInTheDocument();
    expect(screen.queryByText('docker run ...')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /reveal/i }));

    expect(screen.getByText('docker run ...')).toBeInTheDocument();
  });

  it('shows a revealedNote instead, when there is no secondStep', async () => {
    const user = userEvent.setup();
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        revealedNote='Keep it somewhere safe.'
        onClose={vi.fn()}
      />,
    );

    expect(screen.queryByText('Keep it somewhere safe.')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /reveal/i }));

    expect(screen.getByText('Keep it somewhere safe.')).toBeInTheDocument();
  });

  it('shows the default footer note, or a custom one', () => {
    renderWithI18n(
      <SecretRevealDialog
        open
        title='Agent created'
        description='Copy the secret now.'
        secret='sk-super-secret'
        secretLabel='agent-1'
        footerNote='This secret only works for 24h.'
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByText('This secret only works for 24h.')).toBeInTheDocument();
  });
});
