import { describe, it, expect } from 'vitest';
import { GrpcProjectMapper } from './grpc-project.mapper';
import type { Project } from '@/generated/scylla/project/v1/project.ts';

describe('GrpcProjectMapper.toDomain', () => {
  it('unwraps the id and carries name/description through', () => {
    const project: Project = {
      projectId: { value: 'project-1' },
      organizationId: { value: 'org-1' },
      name: 'web',
      description: 'the web app',
      isActive: true,
    };
    expect(GrpcProjectMapper.toDomain(project)).toEqual({
      id: 'project-1',
      name: 'web',
      description: 'the web app',
    });
  });

  it('defaults to an empty id when the wrapper is absent', () => {
    const project: Project = {
      projectId: undefined,
      organizationId: { value: 'org-1' },
      name: 'web',
      description: '',
      isActive: true,
    };
    expect(GrpcProjectMapper.toDomain(project).id).toBe('');
  });
});

describe('GrpcProjectMapper.toDomainList', () => {
  it('maps every project and carries the pagination metadata through', () => {
    const pagination = { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false };
    const result = GrpcProjectMapper.toDomainList({
      projects: [
        {
          projectId: { value: 'project-1' },
          organizationId: { value: 'org-1' },
          name: 'web',
          description: '',
          isActive: true,
        },
      ],
      pagination,
    });
    expect(result.projects).toEqual([{ id: 'project-1', name: 'web', description: '' }]);
    expect(result.pagination).toBe(pagination);
  });
});
