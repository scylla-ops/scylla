import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { findTooltip, render } from '@/test/render.svelte.ts';
import CopyableText from './CopyableText.svelte';

const writeText = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  writeText.mockClear();
  vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
});

afterEach(() => vi.useRealTimers());

describe('CopyableText', () => {
  it('shows the value as-is by default', () => {
    render(CopyableText, { value: 'job-abcdef123456' });

    expect(screen.getByText('job-abcdef123456')).toBeInTheDocument();
  });

  it('truncates the displayed text but copies the whole value', async () => {
    render(CopyableText, { value: 'job-abcdef123456', truncate: 7 });

    expect(screen.getByText('job-abc...')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Copy' }));

    expect(writeText).toHaveBeenCalledWith('job-abcdef123456');
  });

  it('renders a caller-supplied display instead of the value', () => {
    render(CopyableText, { value: 'https://hooks.example/abc', display: 'webhook' });

    expect(screen.getByText('webhook')).toBeInTheDocument();
    expect(screen.queryByText('https://hooks.example/abc')).toBeNull();
  });

  it('switches the button to "Copied!" and back after two seconds', async () => {
    vi.useFakeTimers();
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(CopyableText, { value: 'token' });

    await user.click(screen.getByRole('button', { name: 'Copy' }));
    expect(screen.getByRole('button', { name: 'Copied!' })).toBeInTheDocument();

    await vi.advanceTimersByTimeAsync(2000);

    await waitFor(() => expect(screen.getByRole('button', { name: 'Copy' })).toBeInTheDocument());
  });

  it('keeps the confirmation for the full delay when copied twice in a row', async () => {
    vi.useFakeTimers();
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(CopyableText, { value: 'token' });

    await user.click(screen.getByRole('button', { name: 'Copy' }));
    await vi.advanceTimersByTimeAsync(1500);
    await user.click(screen.getByRole('button', { name: 'Copied!' }));

    // The first timer must not fire 500 ms into the second copy.
    await vi.advanceTimersByTimeAsync(1000);

    expect(screen.getByRole('button', { name: 'Copied!' })).toBeInTheDocument();
  });

  it('uses a custom copy label when given', () => {
    render(CopyableText, { value: 'token', copyLabel: 'Copy token' });

    expect(screen.getByRole('button', { name: 'Copy token' })).toBeInTheDocument();
  });

  it('does not wrap the text in a tooltip trigger unless asked to', () => {
    render(CopyableText, { value: 'token' });

    expect(screen.getAllByRole('button')).toHaveLength(1);
  });

  it('reveals the full value on hover when showFullOnHover is set', async () => {
    render(CopyableText, { value: 'job-abcdef123456', truncate: 7, showFullOnHover: true });

    await userEvent.hover(screen.getByRole('button', { name: 'job-abc...' }));

    expect(await findTooltip()).toHaveTextContent('job-abcdef123456');
  });
});
