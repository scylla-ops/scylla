import { describe, it, expect } from 'vitest';
import { GrpcProjectMemberMapper } from './grpc-project-member.mapper';
import type { ProjectMember } from '@/generated/scylla/project/v1/project.ts';

describe('GrpcProjectMemberMapper.toDomain', () => {
  it('unwraps the user id and carries the username through', () => {
    const member: ProjectMember = { userId: { value: 'user-1' }, username: 'ravenne' };
    expect(GrpcProjectMemberMapper.toDomain(member)).toEqual({ userId: 'user-1', username: 'ravenne' });
  });

  it('defaults to an empty userId when the wrapper is absent', () => {
    const member: ProjectMember = { userId: undefined, username: 'ravenne' };
    expect(GrpcProjectMemberMapper.toDomain(member).userId).toBe('');
  });
});
