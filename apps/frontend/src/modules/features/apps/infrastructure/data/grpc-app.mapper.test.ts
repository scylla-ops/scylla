import { describe, it, expect } from 'vitest';
import { GrpcAppMapper } from './grpc-app.mapper';
import type { App, AppSecret } from '@/generated/scylla/app/v1/app.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

describe('GrpcAppMapper.toDomain', () => {
  it('unwraps ids and formats timestamps', () => {
    const app: App = {
      appId: { value: 'app-1' },
      organizationId: { value: 'org-1' },
      name: 'ci-runner',
      isActive: true,
      createdAt: TS,
      updatedAt: TS,
    };
    expect(GrpcAppMapper.toDomain(app)).toEqual({
      id: 'app-1',
      organizationId: 'org-1',
      name: 'ci-runner',
      isActive: true,
      createdAt: ISO,
      updatedAt: ISO,
    });
  });

  it('defaults to empty ids when the wrappers are absent', () => {
    const app: App = {
      appId: undefined,
      organizationId: undefined,
      name: 'ci-runner',
      isActive: false,
      createdAt: TS,
      updatedAt: TS,
    };
    const domain = GrpcAppMapper.toDomain(app);
    expect(domain.id).toBe('');
    expect(domain.organizationId).toBe('');
  });
});

describe('GrpcAppMapper.secretToDomain', () => {
  it('unwraps ids, formats timestamps, and carries label/enabled through', () => {
    const secret: AppSecret = {
      appSecretId: { value: 'secret-1' },
      appId: { value: 'app-1' },
      label: 'ci-runner-key',
      enabled: true,
      createdAt: TS,
      updatedAt: TS,
    };
    expect(GrpcAppMapper.secretToDomain(secret)).toEqual({
      id: 'secret-1',
      appId: 'app-1',
      label: 'ci-runner-key',
      enabled: true,
      createdAt: ISO,
      updatedAt: ISO,
    });
  });
});
