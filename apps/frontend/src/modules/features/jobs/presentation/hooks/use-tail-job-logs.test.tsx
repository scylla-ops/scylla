import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { act, renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useTailJobLogs } from './use-tail-job-logs';
import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { JobLog } from '@/modules/features/jobs/domain/structs/job.struct.ts';

const JOB_ID = 'job-1';

/**
 * A JobLogStream backed by a queue this test controls: `push`/`pushError` add
 * entries and wake the consumer, `end` completes the async iterable. Mirrors
 * what a real gRPC-Web server-stream reader looks like from the hook's side.
 */
const createControllableStream = () => {
  const queue: ScyllaResult<JobLog>[] = [];
  let wake: (() => void) | null = null;
  let ended = false;
  const cancel = vi.fn();

  const wakeUp = () => {
    const w = wake;
    wake = null;
    w?.();
  };

  async function* generator(): AsyncGenerator<ScyllaResult<JobLog>> {
    let i = 0;
    for (;;) {
      if (i < queue.length) {
        yield queue[i++];
        continue;
      }
      if (ended) return;
      await new Promise<void>(resolve => {
        wake = resolve;
      });
    }
  }

  const logAt = (line: string): JobLog => ({
    id: String(queue.length),
    jobId: JOB_ID,
    nodeId: '',
    stream: 'stdout',
    line,
    timestamp: '2026-01-01T00:00:00.000Z',
  });

  return {
    stream: { logs: generator(), cancel },
    push: (line: string) => {
      queue.push(ScyllaResult.success(logAt(line)));
      wakeUp();
    },
    pushError: (err: ScyllaError) => {
      queue.push(ScyllaResult.error(err));
      wakeUp();
    },
    end: () => {
      ended = true;
      wakeUp();
    },
    cancel,
  };
};

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const wrapperFor = (repository: JobsRepository) => {
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <DependenciesProvider registry={{ jobs: { jobsRepository: repository } }}>
      {children}
    </DependenciesProvider>
  );
  return Wrapper;
};

describe('useTailJobLogs', () => {
  it('does not open a stream for an empty jobId', () => {
    const tailLogs = vi.fn();
    const repository = { tailLogs } as unknown as JobsRepository;
    renderHook(() => useTailJobLogs(''), { wrapper: wrapperFor(repository) });
    expect(tailLogs).not.toHaveBeenCalled();
  });

  it('opens the stream scoped to the job (and node, when given)', () => {
    const { stream } = createControllableStream();
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.success(stream));
    const repository = { tailLogs } as unknown as JobsRepository;
    renderHook(() => useTailJobLogs(JOB_ID, 'checkout'), { wrapper: wrapperFor(repository) });
    expect(tailLogs).toHaveBeenCalledWith(JOB_ID, 'checkout');
  });

  it('sets isError/error and never loads when the repository refuses to open the stream', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'PERMISSION_DENIED' } });
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.error(error));
    const repository = { tailLogs } as unknown as JobsRepository;
    const { result } = renderHook(() => useTailJobLogs(JOB_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
    expect(result.current.isLoading).toBe(false);
  });

  it('buffers incoming lines and flushes them together on the next tick, not one setState per line', async () => {
    const { stream, push } = createControllableStream();
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.success(stream));
    const repository = { tailLogs } as unknown as JobsRepository;
    const { result } = renderHook(() => useTailJobLogs(JOB_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      push('line one');
      push('line two');
      await wait(200); // cross the 150ms flush interval
    });

    expect(result.current.logString).toBe('line one\nline two');
  });

  it('does the final flush once the stream ends, without waiting for the next tick', async () => {
    const { stream, push, end } = createControllableStream();
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.success(stream));
    const repository = { tailLogs } as unknown as JobsRepository;
    const { result } = renderHook(() => useTailJobLogs(JOB_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    push('last line');
    end();

    // No `wait(200)` here: the final flush after the loop exits must not
    // depend on the periodic timer having ticked.
    await waitFor(() => expect(result.current.logString).toBe('last line'));
  });

  it('skips a per-line error entry (logged, not surfaced as a document line) and keeps the rest', async () => {
    const { stream, push, pushError, end } = createControllableStream();
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.success(stream));
    const repository = { tailLogs } as unknown as JobsRepository;
    const { result } = renderHook(() => useTailJobLogs(JOB_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    push('good line 1');
    pushError(new ScyllaError('malformed entry'));
    push('good line 2');
    end();

    await waitFor(() => expect(result.current.logString).toBe('good line 1\ngood line 2'));
  });

  it('cancels the stream on unmount', async () => {
    const { stream, cancel } = createControllableStream();
    const tailLogs = vi.fn().mockReturnValue(ScyllaResult.success(stream));
    const repository = { tailLogs } as unknown as JobsRepository;
    const { unmount, result } = renderHook(() => useTailJobLogs(JOB_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    unmount();

    expect(cancel).toHaveBeenCalled();
  });

  it('resets to a fresh, loading state when jobId changes (new job, new stream)', async () => {
    const first = createControllableStream();
    const second = createControllableStream();
    const tailLogs = vi
      .fn()
      .mockReturnValueOnce(ScyllaResult.success(first.stream))
      .mockReturnValueOnce(ScyllaResult.success(second.stream));
    const repository = { tailLogs } as unknown as JobsRepository;

    const { result, rerender } = renderHook(({ jobId }: { jobId: string }) => useTailJobLogs(jobId), {
      wrapper: wrapperFor(repository),
      initialProps: { jobId: 'job-1' },
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    first.push('from job 1');
    first.end();
    await waitFor(() => expect(result.current.logString).toBe('from job 1'));

    rerender({ jobId: 'job-2' });

    expect(result.current.logString).toBe('');
    expect(first.cancel).toHaveBeenCalled();
    expect(tailLogs).toHaveBeenLastCalledWith('job-2', undefined);
  });
});
