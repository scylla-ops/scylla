import { describe, it, expect } from 'vitest';
import { GrpcUserMapper } from './grpc-user.mapper';
import type { User } from '@/generated/scylla/user/v1/user.ts';

const TS = { seconds: 1735689600n, nanos: 0 };
const ISO = '2025-01-01T00:00:00.000Z';

const baseUser = (overrides: Partial<User> = {}): User => ({
  userId: { value: 'user-1' },
  username: 'ravenne',
  email: undefined,
  isActive: true,
  createdAt: TS,
  updatedAt: TS,
  ...overrides,
});

describe('GrpcUserMapper.toDomain', () => {
  it('unwraps the id and formats the timestamp', () => {
    expect(GrpcUserMapper.toDomain(baseUser())).toEqual({
      username: 'ravenne',
      userId: 'user-1',
      createdAt: ISO,
    });
  });

  it('defaults to an empty userId when the wrapper is absent', () => {
    expect(GrpcUserMapper.toDomain(baseUser({ userId: undefined })).userId).toBe('');
  });
});

describe('GrpcUserMapper.toDomainList', () => {
  it('maps every user and carries the pagination metadata through', () => {
    const pagination = { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false };
    const result = GrpcUserMapper.toDomainList({ users: [baseUser()], pagination });
    expect(result.items).toEqual([{ username: 'ravenne', userId: 'user-1', createdAt: ISO }]);
    expect(result.pagination).toBe(pagination);
  });
});
