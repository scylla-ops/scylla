import { describe, it, expect } from 'vitest';
import { GrpcOrganizationMemberMapper } from './grpc-organization-member.mapper';
import type { OrganizationMember } from '@/generated/scylla/organization/v1/organization.ts';

describe('GrpcOrganizationMemberMapper.toDomain', () => {
  it('unwraps the user id and carries the username through', () => {
    const member: OrganizationMember = { userId: { value: 'user-1' }, username: 'ravenne' };
    expect(GrpcOrganizationMemberMapper.toDomain(member)).toEqual({ userId: 'user-1', username: 'ravenne' });
  });

  it('defaults to an empty userId when the wrapper is absent', () => {
    const member: OrganizationMember = { userId: undefined, username: 'ravenne' };
    expect(GrpcOrganizationMemberMapper.toDomain(member).userId).toBe('');
  });
});
