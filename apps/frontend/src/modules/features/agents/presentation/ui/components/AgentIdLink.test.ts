import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import AgentIdLink from './AgentIdLink.svelte';
import AgentIdLinkInCard from './AgentIdLink.fixture.svelte';

const toastSuccess = vi.fn();
const toastError = vi.fn();
vi.mock('sonner', () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccess(...args),
    error: (...args: unknown[]) => toastError(...args),
  },
}));

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
  toastSuccess.mockClear();
  toastError.mockClear();
  writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
});

describe('AgentIdLink', () => {
  it('shows the full id by default', () => {
    render(AgentIdLink, { id: 'agent-abcdefgh-1234' });
    expect(screen.getByText('agent-abcdefgh-1234')).toBeInTheDocument();
  });

  it('truncates when the id is longer than the given length', () => {
    render(AgentIdLink, { id: 'agent-abcdefgh-1234', truncate: 8 });
    expect(screen.getByText('agent-ab…')).toBeInTheDocument();
  });

  it('does not truncate an id already shorter than the limit', () => {
    render(AgentIdLink, { id: 'short-id', truncate: 20 });
    expect(screen.getByText('short-id')).toBeInTheDocument();
  });

  it('copies the full (untruncated) id and toasts on success', async () => {
    render(AgentIdLink, { id: 'agent-abcdefgh-1234', truncate: 8 });
    await fireEvent.click(screen.getByRole('button'));

    expect(writeText).toHaveBeenCalledWith('agent-abcdefgh-1234');
    await vi.waitFor(() => expect(toastSuccess).toHaveBeenCalledWith('Agent id copied'));
  });

  it('toasts an error when the clipboard write is denied, rather than failing silently', async () => {
    writeText.mockRejectedValueOnce(new Error('denied'));

    render(AgentIdLink, { id: 'agent-1' });
    await fireEvent.click(screen.getByRole('button'));

    await vi.waitFor(() =>
      expect(toastError).toHaveBeenCalledWith('Could not copy — select the id manually'),
    );
    expect(toastSuccess).not.toHaveBeenCalled();
  });

  it("stops the click from bubbling up to a card's own handler", async () => {
    const onCardClick = vi.fn();
    render(AgentIdLinkInCard, { id: 'agent-1', onCardClick });

    await fireEvent.click(screen.getByRole('button'));

    expect(onCardClick).not.toHaveBeenCalled();
  });
});
