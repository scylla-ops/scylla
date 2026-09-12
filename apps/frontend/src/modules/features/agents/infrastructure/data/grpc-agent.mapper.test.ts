import { describe, it, expect } from 'vitest';
import { GrpcAgentMapper } from './grpc-agent.mapper';
import type { Agent, AgentHost, AgentStats } from '@/generated/scylla/agent/v1/agent_admin.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

const baseAgent = (overrides: Partial<Agent> = {}): Agent => ({
  agentId: { value: 'agent-1' },
  organizationId: { value: 'org-1' },
  name: 'runner-1',
  isActive: true,
  connected: true,
  lastSeen: TS,
  inFlight: 2,
  host: undefined,
  createdAt: TS,
  updatedAt: TS,
  ...overrides,
});

describe('GrpcAgentMapper.toDomain', () => {
  it('unwraps ids, formats timestamps, and carries scalar fields through', () => {
    const domain = GrpcAgentMapper.toDomain(baseAgent());
    expect(domain.id).toBe('agent-1');
    expect(domain.organizationId).toBe('org-1');
    expect(domain.name).toBe('runner-1');
    expect(domain.isActive).toBe(true);
    expect(domain.connected).toBe(true);
    expect(domain.inFlight).toBe(2);
    expect(domain.lastSeen).toBe(ISO);
    expect(domain.createdAt).toBe(ISO);
    expect(domain.updatedAt).toBe(ISO);
  });

  it('host is null when the agent has never said hello', () => {
    expect(GrpcAgentMapper.toDomain(baseAgent({ host: undefined })).host).toBeNull();
  });

  describe('host', () => {
    const host = (overrides: Partial<AgentHost> = {}): AgentHost => ({
      version: '1.2.3',
      os: 'linux',
      arch: 'amd64',
      hostname: 'runner-box',
      cpuCount: 8,
      totalMemoryMb: 16000n,
      reportedAt: TS,
      ...overrides,
    });

    it('maps a fully-reported host', () => {
      const domain = GrpcAgentMapper.toDomain(baseAgent({ host: host() })).host;
      expect(domain).toEqual({
        version: '1.2.3',
        os: 'linux',
        arch: 'amd64',
        hostname: 'runner-box',
        cpuCount: 8,
        totalMemoryMb: 16000,
        reportedAt: ISO,
      });
    });

    it('a cpuCount of 0 (never reported, not "zero cores") reads as null', () => {
      expect(GrpcAgentMapper.toDomain(baseAgent({ host: host({ cpuCount: 0 }) })).host?.cpuCount).toBeNull();
    });

    it('a totalMemoryMb of 0 reads as null the same way', () => {
      expect(
        GrpcAgentMapper.toDomain(baseAgent({ host: host({ totalMemoryMb: 0n }) })).host?.totalMemoryMb,
      ).toBeNull();
    });

    it('a positive totalMemoryMb converts the bigint to a plain number', () => {
      const totalMemoryMb = GrpcAgentMapper.toDomain(
        baseAgent({ host: host({ totalMemoryMb: 32000n }) }),
      ).host?.totalMemoryMb;
      expect(totalMemoryMb).toBe(32000);
      expect(typeof totalMemoryMb).toBe('number');
    });
  });
});

describe('GrpcAgentMapper.statsToDomain', () => {
  const baseStats = (overrides: Partial<AgentStats> = {}): AgentStats => ({
    total: 100n,
    pending: 1n,
    running: 2n,
    completed: 90n,
    failed: 5n,
    cancelled: 2n,
    orphaned: 0n,
    lastRunAt: TS,
    daily: [],
    medianDurationMs: undefined,
    p95DurationMs: undefined,
    ...overrides,
  });

  it('converts every bigint counter to a plain number', () => {
    const domain = GrpcAgentMapper.statsToDomain(baseStats());
    expect(domain).toMatchObject({
      total: 100,
      pending: 1,
      running: 2,
      completed: 90,
      failed: 5,
      cancelled: 2,
      orphaned: 0,
      lastRunAt: ISO,
    });
  });

  it('an unset medianDurationMs/p95DurationMs reads as null, not 0 (no job has run yet vs. a 0ms run)', () => {
    const domain = GrpcAgentMapper.statsToDomain(baseStats());
    expect(domain.medianDurationMs).toBeNull();
    expect(domain.p95DurationMs).toBeNull();
  });

  it('a set duration converts from bigint to a plain number', () => {
    const domain = GrpcAgentMapper.statsToDomain(
      baseStats({ medianDurationMs: 1500n, p95DurationMs: 4200n }),
    );
    expect(domain.medianDurationMs).toBe(1500);
    expect(domain.p95DurationMs).toBe(4200);
  });

  it('maps each daily bucket, including its own optional medianDurationMs', () => {
    const domain = GrpcAgentMapper.statsToDomain(
      baseStats({
        daily: [
          { day: TS, completed: 3n, failed: 1n, cancelled: 0n, orphaned: 0n, medianDurationMs: 800n },
          { day: TS, completed: 0n, failed: 0n, cancelled: 0n, orphaned: 0n, medianDurationMs: undefined },
        ],
      }),
    );
    expect(domain.daily).toEqual([
      { day: ISO, completed: 3, failed: 1, cancelled: 0, orphaned: 0, medianDurationMs: 800 },
      { day: ISO, completed: 0, failed: 0, cancelled: 0, orphaned: 0, medianDurationMs: null },
    ]);
  });
});
