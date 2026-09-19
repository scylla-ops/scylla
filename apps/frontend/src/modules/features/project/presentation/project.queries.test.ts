// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ProjectEntity } from '../domain/entities/project.entity.ts';
import type { ProjectRepository } from '../domain/repository/project.repository.ts';
import {
  canListProjects,
  invalidateProjectMembers,
  projectLookupQueries,
  projectMutations,
  projectQueries,
  PROJECTS_LOOKUP_PAGE,
  PROJECTS_QUERY_KEY,
  PROJECTS_QUERY_ROOT,
  PROJECT_MEMBERS_QUERY_KEY,
} from './project.queries.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'web',
  description: 'the web app',
  ...overrides,
});

const page = (projects: ProjectEntity[]) => ({
  projects,
  pagination: { page: 1, pageSize: 10, totalCount: projects.length, totalPages: 1, hasNext: false, hasPrevious: false },
});

const withRepository = (overrides: Partial<ProjectRepository> = {}) => {
  const repository = {
    getByOrganizationId: vi.fn().mockResolvedValue(ScyllaResult.success(page([project()]))),
    listMembers: vi.fn().mockResolvedValue(ScyllaResult.success([])),
    create: vi.fn().mockResolvedValue(ScyllaResult.success(project())),
    update: vi.fn().mockResolvedValue(ScyllaResult.success(project())),
    delete: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    ...overrides,
  } as unknown as ProjectRepository;

  setDependencyRegistry({ project: { projectRepository: repository } });
  return repository;
};

const grantEverything = () =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const grantNothing = () => usePermissionsStore.setState({ permissions: { scopes: [] } });
const forgetPermissions = () => usePermissionsStore.setState({ permissions: null });

let queryClient: QueryClient;

beforeEach(() => {
  toastSuccess.mockClear();
  grantEverything();
  queryClient = new QueryClient();
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
  forgetPermissions();
});

describe('canListProjects', () => {
  it('denies while the permissions are still unknown', () => {
    forgetPermissions();
    expect(canListProjects('org-1')).toBe(false);
  });

  it('denies without LIST_PROJECTS_BY_ORGANIZATION', () => {
    grantNothing();
    expect(canListProjects('org-1')).toBe(false);
  });

  it('denies without an organization', () => {
    expect(canListProjects(null)).toBe(false);
  });
});

describe('projectQueries.byOrganization', () => {
  it('fetches the first page for the given organization', async () => {
    const repository = withRepository();

    const result = await runQueryFn(projectQueries.byOrganization('org-1'));

    expect(repository.getByOrganizationId).toHaveBeenCalledWith('org-1', {
      page: 1,
      pageSize: expect.any(Number) as number,
    });
    expect(result.projects).toHaveLength(1);
  });

  it('never asks for a list it may not read', () => {
    grantNothing();
    withRepository();
    expect(projectQueries.byOrganization('org-1').enabled).toBe(false);
  });

  it('is disabled without an organization', () => {
    withRepository();
    expect(projectQueries.byOrganization(null).enabled).toBe(false);
  });

  it('keys each page separately, so a page change is a new cache entry', () => {
    withRepository();
    const first = projectQueries.byOrganization('org-1', { page: 1, pageSize: 10 }).queryKey;
    const second = projectQueries.byOrganization('org-1', { page: 2, pageSize: 10 }).queryKey;

    expect(first).not.toEqual(second);
  });
});

describe('projectQueries.lookup', () => {
  it('asks for the whole list in one page', async () => {
    const repository = withRepository();

    await runQueryFn(projectQueries.lookup('org-1'));

    expect(repository.getByOrganizationId).toHaveBeenCalledWith('org-1', PROJECTS_LOOKUP_PAGE);
  });

  it('shares its cache entry with the fan-out lookup for the same organization', () => {
    withRepository();

    expect(projectQueries.lookup('org-1').queryKey).toEqual(
      PROJECTS_QUERY_KEY('org-1', PROJECTS_LOOKUP_PAGE),
    );
  });

  it('is disabled without an organization', () => {
    withRepository();
    expect(projectQueries.lookup(null).enabled).toBe(false);
  });
});

describe('projectLookupQueries', () => {
  it('skips the organizations the caller may not read rather than collecting denials', () => {
    usePermissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.ORGANIZATION,
            scopeId: 'org-1',
            access: { kind: 'restricted', permissions: [Permission.LIST_PROJECTS_BY_ORGANIZATION] },
          },
        ],
      },
    });
    withRepository();

    const { queries } = projectLookupQueries(['org-1', 'org-2']);

    expect(queries).toHaveLength(1);
    expect(queries[0].queryKey).toEqual(PROJECTS_QUERY_KEY('org-1', PROJECTS_LOOKUP_PAGE));
  });

  it('queries nothing when disabled', () => {
    withRepository();
    expect(projectLookupQueries(['org-1'], false).queries).toHaveLength(0);
  });

  it('combines results into a projectId -> {name, organizationId} map', () => {
    withRepository();
    const { combine } = projectLookupQueries(['org-1', 'org-2']);

    const map = combine([
      { data: page([project({ id: 'p-1', name: 'web' })]) },
      { data: page([project({ id: 'p-2', name: 'api' })]) },
    ]);

    expect(map.get('p-1')).toEqual({ name: 'web', organizationId: 'org-1' });
    expect(map.get('p-2')).toEqual({ name: 'api', organizationId: 'org-2' });
  });

  it('keeps each result attached to the organization it came from when some are skipped', () => {
    usePermissionsStore.setState({
      permissions: {
        scopes: [
          {
            scope: PermissionScope.ORGANIZATION,
            scopeId: 'org-2',
            access: { kind: 'restricted', permissions: [Permission.LIST_PROJECTS_BY_ORGANIZATION] },
          },
        ],
      },
    });
    withRepository();

    const { combine } = projectLookupQueries(['org-1', 'org-2']);
    const map = combine([{ data: page([project({ id: 'p-2' })]) }]);

    // The single result is org-2's, not org-1's — the indexes follow the
    // *readable* ids, not the ones asked for.
    expect(map.get('p-2')?.organizationId).toBe('org-2');
  });
});

describe('projectQueries.members', () => {
  it('lists members for the given project', async () => {
    const repository = withRepository();

    await runQueryFn(projectQueries.members('project-1'));

    expect(repository.listMembers).toHaveBeenCalledWith('project-1');
  });

  it('is disabled without a project', () => {
    withRepository();
    expect(projectQueries.members(null).enabled).toBe(false);
  });

  it("invalidates exactly this project's members key", () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    invalidateProjectMembers('project-1');

    expect(invalidate).toHaveBeenCalledWith({
      queryKey: PROJECT_MEMBERS_QUERY_KEY('project-1'),
      exact: true,
    });
  });

  it('invalidating without a project is a no-op', () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    invalidateProjectMembers(null);

    expect(invalidate).not.toHaveBeenCalled();
  });
});

describe('the mutations', () => {
  it('create scopes the project to the organization, toasts and invalidates the projects root', async () => {
    const repository = withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    await runMutationFn(projectMutations.create(), {
      name: 'web',
      organizationId: 'org-1',
      description: 'd',
    });
    runOnSuccess(projectMutations.create(), project(), { name: 'web', organizationId: 'org-1' });

    expect(repository.create).toHaveBeenCalledWith('web', 'org-1', 'd');
    expect(toastSuccess).toHaveBeenCalledWith('Project created');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: PROJECTS_QUERY_ROOT });
  });

  it('update sends only what changed and invalidates the projects root', async () => {
    const repository = withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    await runMutationFn(projectMutations.update(), { projectId: 'project-9', name: 'renamed' });
    runOnSuccess(projectMutations.update(), project(), { projectId: 'project-9' });

    expect(repository.update).toHaveBeenCalledWith('project-9', 'renamed', undefined);
    expect(toastSuccess).toHaveBeenCalledWith('Project updated');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: PROJECTS_QUERY_ROOT });
  });

  it('remove deletes and invalidates the projects root', async () => {
    const repository = withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    await runMutationFn(projectMutations.remove(), 'project-9');
    runOnSuccess(projectMutations.remove(), undefined, 'project-9');

    expect(repository.delete).toHaveBeenCalledWith('project-9');
    expect(toastSuccess).toHaveBeenCalledWith('Project deleted');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: PROJECTS_QUERY_ROOT });
  });
});
