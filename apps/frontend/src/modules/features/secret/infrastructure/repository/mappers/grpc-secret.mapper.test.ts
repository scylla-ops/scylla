import { describe, it, expect } from 'vitest';
import { GrpcSecretMapper } from './grpc-secret.mapper';
import type { Secret } from '@/generated/scylla/secret/v1/secret.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

describe('GrpcSecretMapper.toDomain', () => {
  it('unwraps ids, formats timestamps, never carries the value (metadata only)', () => {
    const secret: Secret = {
      secretId: { value: 'secret-1' },
      projectId: { value: 'project-1' },
      name: 'DATABASE_URL',
      description: 'prod db',
      createdAt: TS,
      updatedAt: TS,
    };
    const domain = GrpcSecretMapper.toDomain(secret);
    expect(domain).toEqual({
      id: 'secret-1',
      projectId: 'project-1',
      name: 'DATABASE_URL',
      description: 'prod db',
      createdAt: ISO,
      updatedAt: ISO,
    });
    expect(domain).not.toHaveProperty('value');
  });

  it('defaults to empty ids when the wrappers are absent', () => {
    const secret: Secret = {
      secretId: undefined,
      projectId: undefined,
      name: 'DATABASE_URL',
      description: '',
      createdAt: TS,
      updatedAt: TS,
    };
    const domain = GrpcSecretMapper.toDomain(secret);
    expect(domain.id).toBe('');
    expect(domain.projectId).toBe('');
  });
});
