import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/svelte';
import { render, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobLog } from '../../../../domain/structs/job.struct.ts';
import type { JobsRepository } from '../../../../domain/repository/jobs.repository.ts';
import JobLogDisplay from './JobLogDisplay.svelte';

const logLine = (line: string): JobLog => ({
  id: line,
  jobId: 'job-1',
  nodeId: '',
  stream: 'stdout',
  line,
  timestamp: '2026-01-01T00:00:00.000Z',
});

const streamOf = (lines: string[]) => ({
  // eslint-disable-next-line @typescript-eslint/require-await
  logs: (async function* () {
    for (const line of lines) yield ScyllaResult.success(logLine(line));
  })(),
  cancel: vi.fn(),
});

let restore: (() => void) | null = null;

const install = (tailLogs: JobsRepository['tailLogs']) => {
  restore = withRegistry({ jobs: { jobsRepository: { tailLogs } as unknown as JobsRepository } });
};

const editor = () => document.querySelector<HTMLElement>('.cm-editor');

beforeEach(() => {
  restore = null;
});

afterEach(() => restore?.());

describe('JobLogDisplay', () => {
  it('waits rather than opening an editor when there is no job to tail', () => {
    const tailLogs = vi.fn();
    install(tailLogs);

    render(JobLogDisplay, { jobId: '' });

    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(tailLogs).not.toHaveBeenCalled();
    expect(editor()).toBeNull();
  });

  it('shows an error message when the stream could not be opened', async () => {
    install(() => ScyllaResult.error(new ScyllaError('no stream')));

    render(JobLogDisplay, { jobId: 'job-1' });

    expect(await screen.findByText('Error loading logs...')).toBeInTheDocument();
  });

  it('writes the streamed lines into the editor document', async () => {
    install(() => ScyllaResult.success(streamOf(['first line', 'second line'])));

    render(JobLogDisplay, { jobId: 'job-1' });

    // The end of the stream flushes at once.
    await waitFor(() => expect(editor()?.textContent).toContain('first line'));
    expect(editor()?.textContent).toContain('second line');
  });

  it('scopes the stream to the node when the caller names one', async () => {
    const tailLogs = vi.fn(() => ScyllaResult.success(streamOf([])));
    install(tailLogs);

    render(JobLogDisplay, { jobId: 'job-1', nodeId: 'build' });

    await waitFor(() => expect(tailLogs).toHaveBeenCalledWith('job-1', 'build'));
  });

  it('grows only to the height the caller measured for it', async () => {
    install(() => ScyllaResult.success(streamOf(['a line'])));

    render(JobLogDisplay, { jobId: 'job-1', maxHeight: 300 });

    await waitFor(() => expect(editor()).not.toBeNull());
    expect(editor()?.style.maxHeight || getComputedStyle(editor()!).maxHeight).toContain('300px');
  });

  it('falls back to a fixed height when the caller measured none', async () => {
    install(() => ScyllaResult.success(streamOf(['a line'])));

    render(JobLogDisplay, { jobId: 'job-1' });

    await waitFor(() => expect(editor()).not.toBeNull());
    expect(getComputedStyle(editor()!).maxHeight).toContain('448px');
  });
});
