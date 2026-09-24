import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { setDependencyRegistry } from '@platform/di';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobLog } from '../../domain/structs/job.struct.ts';
import type { JobsRepository } from '../../domain/repository/jobs.repository.ts';
import { createTailJobLogs } from '../tail-job-logs.svelte.ts';

const JOB_ID = 'job-1';

/** A log stream this test drives: `push`, `pushError`, `end`. */
const createControllableStream = () => {
  const queue: ScyllaResult<JobLog>[] = [];
  let wake: (() => void) | null = null;
  let ended = false;
  const cancel = vi.fn();

  const wakeUp = () => {
    const resume = wake;
    wake = null;
    resume?.();
  };

  async function* generator(): AsyncGenerator<ScyllaResult<JobLog>> {
    let index = 0;
    for (;;) {
      if (index < queue.length) {
        yield queue[index++];
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
    pushError: (error: ScyllaError) => {
      queue.push(ScyllaResult.error(error));
      wakeUp();
    },
    end: () => {
      ended = true;
      wakeUp();
    },
    cancel,
  };
};

const settle = async (ms = 200) => {
  await Promise.resolve();
  await new Promise(resolve => setTimeout(resolve, ms));
  flushSync();
};

let tailLogs: ReturnType<typeof vi.fn<JobsRepository['tailLogs']>>;

const install = (repository: Partial<JobsRepository>) =>
  setDependencyRegistry({ jobs: { jobsRepository: repository as JobsRepository } });

/** Reads through a getter: `$effect.root` returns its teardown, not the callback's value. */
const start = (nodeId?: () => string | undefined) => {
  let tail: ReturnType<typeof createTailJobLogs> | undefined;

  const cleanup = $effect.root(() => {
    tail = createTailJobLogs({ jobId: () => JOB_ID, nodeId });
    flushSync();
  });

  return { tail: () => tail!, cleanup };
};

beforeEach(() => {
  tailLogs = vi.fn<JobsRepository['tailLogs']>();
});

afterEach(() => setDependencyRegistry(null));

describe('createTailJobLogs', () => {
  it('does not open a stream for an empty jobId', () => {
    install({ tailLogs });

    const cleanup = $effect.root(() => {
      createTailJobLogs({ jobId: () => '' });
      flushSync();
    });
    cleanup();

    expect(tailLogs).not.toHaveBeenCalled();
  });

  it('opens the stream scoped to the job, and to the node when given one', () => {
    const { stream } = createControllableStream();
    tailLogs.mockReturnValue(ScyllaResult.success(stream));
    install({ tailLogs });

    const cleanup = $effect.root(() => {
      createTailJobLogs({ jobId: () => JOB_ID, nodeId: () => 'build' });
      flushSync();
    });
    cleanup();

    expect(tailLogs).toHaveBeenCalledWith(JOB_ID, 'build');
  });

  it('reports the failure and stops loading when the stream cannot be opened', () => {
    const error = new ScyllaError('nope');
    tailLogs.mockReturnValue(ScyllaResult.error(error));
    install({ tailLogs });

    const cleanup = $effect.root(() => {
      const tail = createTailJobLogs({ jobId: () => JOB_ID });
      flushSync();

      expect(tail.isError).toBe(true);
      expect(tail.error).toBe(error);
      expect(tail.isLoading).toBe(false);
    });
    cleanup();
  });

  it('buffers a burst of lines and publishes them together, not once per line', async () => {
    const controller = createControllableStream();
    tailLogs.mockReturnValue(ScyllaResult.success(controller.stream));
    install({ tailLogs });

    const { tail, cleanup } = start();

    controller.push('first');
    controller.push('second');
    await settle();

    expect(tail().text).toBe('first\nsecond');
    cleanup();
  });

  it('does a final flush when the stream ends, rather than losing the last lines between ticks', async () => {
    const controller = createControllableStream();
    tailLogs.mockReturnValue(ScyllaResult.success(controller.stream));
    install({ tailLogs });

    const { tail, cleanup } = start();

    controller.push('only line');
    controller.end();
    // Shorter than the flush interval: the end of the stream publishes this.
    await settle(10);

    expect(tail().text).toBe('only line');
    cleanup();
  });

  it('logs a per-line error instead of writing it into the document', async () => {
    const controller = createControllableStream();
    tailLogs.mockReturnValue(ScyllaResult.success(controller.stream));
    install({ tailLogs });

    const failure = new ScyllaError('one bad line');
    const logged = vi.spyOn(failure, 'log').mockImplementation(() => failure);

    const { tail, cleanup } = start();

    controller.push('before');
    controller.pushError(failure);
    controller.push('after');
    await settle();

    expect(tail().text).toBe('before\nafter');
    expect(logged).toHaveBeenCalled();
    cleanup();
  });

  it('cancels the stream when the view goes away', () => {
    const controller = createControllableStream();
    tailLogs.mockReturnValue(ScyllaResult.success(controller.stream));
    install({ tailLogs });

    const cleanup = $effect.root(() => {
      createTailJobLogs({ jobId: () => JOB_ID });
      flushSync();
    });
    cleanup();

    expect(controller.cancel).toHaveBeenCalled();
  });

  it('opens a fresh stream, from a clean document, when the job changes', () => {
    const first = createControllableStream();
    const second = createControllableStream();
    tailLogs
      .mockReturnValueOnce(ScyllaResult.success(first.stream))
      .mockReturnValueOnce(ScyllaResult.success(second.stream));
    install({ tailLogs });

    const cleanup = $effect.root(() => {
      let jobId = $state(JOB_ID);
      const tail = createTailJobLogs({ jobId: () => jobId });
      flushSync();

      jobId = 'job-2';
      flushSync();

      expect(first.cancel).toHaveBeenCalled();
      expect(tailLogs).toHaveBeenLastCalledWith('job-2', undefined);
      // Cleared: the previous job's output must not lead the new one's.
      expect(tail.text).toBe('');
    });
    cleanup();
  });
});
