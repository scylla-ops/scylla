import { describe, it, expect } from 'vitest';
import { GrpcOrganizationMapper } from './grpc-organization.mapper';
import type { Organization } from '@/generated/scylla/organization/v1/organization.ts';

describe('GrpcOrganizationMapper.toDomain', () => {
  it('unwraps the id and carries name/description through', () => {
    const org: Organization = {
      organizationId: { value: 'org-1' },
      name: 'Acme',
      description: 'Acme Corp',
      isActive: true,
      createdAt: { seconds: 1735689600n, nanos: 0 },
    };
    expect(GrpcOrganizationMapper.toDomain(org)).toEqual({
      id: 'org-1',
      name: 'Acme',
      description: 'Acme Corp',
    });
  });

  it('defaults to an empty id when the wrapper is absent', () => {
    const org: Organization = {
      organizationId: undefined,
      name: 'Acme',
      description: '',
      isActive: true,
      createdAt: { seconds: 1735689600n, nanos: 0 },
    };
    expect(GrpcOrganizationMapper.toDomain(org).id).toBe('');
  });
});
