import { describe, it, expect, vi } from 'vitest';
import { GrpcJobMapper } from './grpc-job.mapper';
import { JobOutcome, NodeOutcome } from '@/generated/scylla/job/v1/job.ts';
import type { Job, JobNode, JobLogEntry } from '@/generated/scylla/job/v1/job.ts';
import { LogStream } from '@/generated/scylla/common/v1/common.ts';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobLogsTailHandleRepo } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import type { JobLog } from '@/modules/features/jobs/domain/structs/job.struct.ts';

const TS = { seconds: 1735689600n, nanos: 0 };

const baseJob = (overrides: Partial<Job> = {}): Job => ({
  jobId: { value: 'job-1' },
  pipelineId: { value: 'pipeline-1' },
  nodeExecutions: [],
  createdAt: TS,
  updatedAt: TS,
  state: { oneofKind: 'pending', pending: {} },
  origin: { oneofKind: undefined },
  ...overrides,
});

describe('GrpcJobMapper.toDomain', () => {
  it('unwraps the ids and formats the timestamps', () => {
    const domain = GrpcJobMapper.toDomain(baseJob());
    expect(domain.id).toBe('job-1');
    expect(domain.pipelineId).toBe('pipeline-1');
    expect(domain.createdAt).toBe('2025-01-01T00:00:00.000Z');
    expect(domain.updatedAt).toBe('2025-01-01T00:00:00.000Z');
  });

  it('a pending job has no startedAt/finishedAt', () => {
    const domain = GrpcJobMapper.toDomain(baseJob({ state: { oneofKind: 'pending', pending: {} } }));
    expect(domain.status).toBe('pending');
    expect(domain.startedAt).toBeUndefined();
    expect(domain.finishedAt).toBeUndefined();
  });

  it('a running job surfaces its startedAt, still no finishedAt', () => {
    const domain = GrpcJobMapper.toDomain(
      baseJob({ state: { oneofKind: 'running', running: { startedAt: TS } } }),
    );
    expect(domain.status).toBe('running');
    expect(domain.startedAt).toBe('2025-01-01T00:00:00.000Z');
    expect(domain.finishedAt).toBeUndefined();
  });

  it.each([
    [JobOutcome.COMPLETED, 'completed'],
    [JobOutcome.FAILED, 'failed'],
    [JobOutcome.CANCELLED, 'cancelled'],
    [JobOutcome.ORPHANED, 'orphaned'],
  ])('a terminal job with outcome %i maps to status "%s"', (outcome, status) => {
    const domain = GrpcJobMapper.toDomain(
      baseJob({ state: { oneofKind: 'terminal', terminal: { outcome, startedAt: TS, finishedAt: TS } } }),
    );
    expect(domain.status).toBe(status);
    expect(domain.startedAt).toBe('2025-01-01T00:00:00.000Z');
    expect(domain.finishedAt).toBe('2025-01-01T00:00:00.000Z');
  });

  it('an UNSPECIFIED outcome, or a state arm newer than this build, maps to "unknown" rather than a guess', () => {
    expect(
      GrpcJobMapper.toDomain(
        baseJob({ state: { oneofKind: 'terminal', terminal: { outcome: JobOutcome.UNSPECIFIED } } }),
      ).status,
    ).toBe('unknown');

    expect(GrpcJobMapper.toDomain(baseJob({ state: { oneofKind: undefined } })).status).toBe('unknown');
  });

  it('a terminal job cancelled before it ever started has no startedAt (only finishedAt)', () => {
    const domain = GrpcJobMapper.toDomain(
      baseJob({
        state: { oneofKind: 'terminal', terminal: { outcome: JobOutcome.CANCELLED, finishedAt: TS } },
      }),
    );
    expect(domain.startedAt).toBeUndefined();
    expect(domain.finishedAt).toBe('2025-01-01T00:00:00.000Z');
  });

  describe('node executions', () => {
    const node = (execution: JobNode['execution'], nodeId = 'node-1'): JobNode => ({
      nodeId: { value: nodeId },
      execution,
    });

    it('maps a pending node with no timestamps', () => {
      const [n] = GrpcJobMapper.toDomain(
        baseJob({ nodeExecutions: [node({ oneofKind: 'pending', pending: {} })] }),
      ).nodeExecutions;
      expect(n).toEqual({ id: 'node-1', state: 'pending', startedAt: undefined, finishedAt: undefined });
    });

    it('maps a running node with its startedAt', () => {
      const [n] = GrpcJobMapper.toDomain(
        baseJob({ nodeExecutions: [node({ oneofKind: 'running', running: { startedAt: TS } })] }),
      ).nodeExecutions;
      expect(n.state).toBe('running');
      expect(n.startedAt).toBe('2025-01-01T00:00:00.000Z');
    });

    it.each([
      [NodeOutcome.COMPLETED, 'completed'],
      [NodeOutcome.FAILED, 'failed'],
      [NodeOutcome.CANCELLED, 'cancelled'],
      [NodeOutcome.SKIPPED, 'skipped'],
    ])('a finished node with outcome %i maps to state "%s"', (outcome, state) => {
      const [n] = GrpcJobMapper.toDomain(
        baseJob({
          nodeExecutions: [
            node({ oneofKind: 'finished', finished: { outcome, startedAt: TS, finishedAt: TS } }),
          ],
        }),
      ).nodeExecutions;
      expect(n.state).toBe(state);
      expect(n.startedAt).toBe('2025-01-01T00:00:00.000Z');
      expect(n.finishedAt).toBe('2025-01-01T00:00:00.000Z');
    });

    it('an execution arm newer than this build maps to "unknown"', () => {
      const [n] = GrpcJobMapper.toDomain(
        baseJob({ nodeExecutions: [node({ oneofKind: undefined })] }),
      ).nodeExecutions;
      expect(n.state).toBe('unknown');
    });

    it('preserves node order and count', () => {
      const domain = GrpcJobMapper.toDomain(
        baseJob({
          nodeExecutions: [
            node({ oneofKind: 'pending', pending: {} }, 'a'),
            node({ oneofKind: 'pending', pending: {} }, 'b'),
          ],
        }),
      );
      expect(domain.nodeExecutions.map(n => n.id)).toEqual(['a', 'b']);
    });
  });
});

describe('GrpcJobMapper.toDomainList', () => {
  it('maps every job and carries the pagination metadata through as-is', () => {
    const pagination = { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false };
    const result = GrpcJobMapper.toDomainList({ jobs: [baseJob()], pagination });
    expect(result.items).toHaveLength(1);
    expect(result.items[0].id).toBe('job-1');
    expect(result.pagination).toBe(pagination);
  });
});

describe('GrpcJobMapper.logEntryToDomain', () => {
  const entry = (overrides: Partial<JobLogEntry> = {}): JobLogEntry => ({
    jobLogId: { value: 'log-1' },
    jobId: { value: 'job-1' },
    nodeId: { value: 'node-1' },
    stream: LogStream.STDOUT,
    line: 'hello',
    timestamp: TS,
    ...overrides,
  });

  it('unwraps ids, resolves the stream, and formats the timestamp', () => {
    const log = GrpcJobMapper.logEntryToDomain(entry());
    expect(log).toEqual({
      id: 'log-1',
      jobId: 'job-1',
      nodeId: 'node-1',
      stream: 'stdout',
      line: 'hello',
      timestamp: '2025-01-01T00:00:00.000Z',
    });
  });

  it('maps STDERR to "stderr"', () => {
    expect(GrpcJobMapper.logEntryToDomain(entry({ stream: LogStream.STDERR })).stream).toBe('stderr');
  });

  it('maps UNSPECIFIED (or a future stream) to an empty string', () => {
    expect(GrpcJobMapper.logEntryToDomain(entry({ stream: LogStream.UNSPECIFIED })).stream).toBe('');
  });
});

describe('GrpcJobMapper.logsToDomainList', () => {
  it('maps every log entry and carries the pagination through', () => {
    const pagination = { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false };
    const result = GrpcJobMapper.logsToDomainList({
      logs: [
        {
          jobLogId: { value: 'log-1' },
          jobId: { value: 'job-1' },
          nodeId: { value: 'node-1' },
          stream: LogStream.STDOUT,
          line: 'hello',
          timestamp: TS,
        },
      ],
      pagination,
    });
    expect(result.items).toHaveLength(1);
    expect(result.pagination).toBe(pagination);
  });
});

describe('GrpcJobMapper.logStreamToDomain', () => {
  it('maps each streamed ScyllaResult<JobLogEntry> to a ScyllaResult<JobLog>, and forwards cancel', async () => {
    const cancel = vi.fn();
    const entry: JobLogEntry = {
      jobLogId: { value: 'log-1' },
      jobId: { value: 'job-1' },
      nodeId: { value: 'node-1' },
      stream: LogStream.STDOUT,
      line: 'hello',
      timestamp: TS,
    };

    async function* responses(): AsyncGenerator<ScyllaResult<JobLogEntry>> {
      await Promise.resolve();
      yield ScyllaResult.success(entry);
    }

    const handle: JobLogsTailHandleRepo = { responses: responses(), cancel };
    const stream = GrpcJobMapper.logStreamToDomain(handle);

    const received: unknown[] = [];
    for await (const result of stream.logs) {
      received.push(result.unwrap());
    }

    expect(received).toEqual([
      { id: 'log-1', jobId: 'job-1', nodeId: 'node-1', stream: 'stdout', line: 'hello', timestamp: '2025-01-01T00:00:00.000Z' },
    ]);
    expect(stream.cancel).toBe(cancel);
  });

  it('passes a per-item error straight through untouched', async () => {
    const error = new ScyllaError('stream broke');
    async function* responses(): AsyncGenerator<ScyllaResult<JobLogEntry>> {
      await Promise.resolve();
      yield ScyllaResult.error(error);
    }

    const stream = GrpcJobMapper.logStreamToDomain({ responses: responses(), cancel: vi.fn() });

    const results: ScyllaResult<JobLog>[] = [];
    for await (const result of stream.logs) {
      results.push(result);
    }

    expect(() => results[0].unwrap()).toThrow('stream broke');
  });
});
