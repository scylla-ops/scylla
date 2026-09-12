import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { useCreateProject } from './useCreateProject';
import { useDeleteProject } from './use-delete-project';
import { useUpdateProject } from './use-update-project';
import { useProjects } from './useProjects';
import { useOrganizationProjects } from './use-organization-projects';
import { useProjectsByOrganizations } from './use-projects-by-organizations';
import { useProjectMembers, PROJECT_MEMBERS_QUERY_KEY } from './use-project-members';
import { PROJECTS_LOOKUP_PAGE, PROJECTS_QUERY_KEY } from './projects.query-keys';
import type { ProjectRepository } from '@/modules/features/project/domain/repository/project.repository.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { ProjectMember } from '@/modules/features/project/domain/structs/project-member.struct.ts';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const ORG_ID = 'org-1';

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'ci-platform',
  description: '',
  ...overrides,
});

const pagination = (overrides: Partial<PaginationInfo> = {}): PaginationInfo => ({
  totalCount: 1,
  page: 1,
  pageSize: 10,
  totalPages: 1,
  hasNext: false,
  hasPrevious: false,
  ...overrides,
});

const member = (overrides: Partial<ProjectMember> = {}): ProjectMember => ({
  userId: 'user-1',
  username: 'ravenne',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<ProjectRepository> = {}) => {
  const getByOrganizationId =
    overrides.getByOrganizationId ??
    vi.fn().mockResolvedValue(ScyllaResult.success({ projects: [project()], pagination: pagination() }));
  const listMembers =
    overrides.listMembers ?? vi.fn().mockResolvedValue(ScyllaResult.success([member()]));
  const create = overrides.create ?? vi.fn().mockResolvedValue(ScyllaResult.success(project()));
  const update = overrides.update ?? vi.fn().mockResolvedValue(ScyllaResult.success(project()));
  const del = overrides.delete ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  const repository: ProjectRepository = {
    getByOrganizationId,
    listMembers,
    create,
    update,
    delete: del,
  };
  return { repository, getByOrganizationId, listMembers, create, update, delete: del };
};

const wrapperFor = (repository: ProjectRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ project: { projectRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

describe('useCreateProject', () => {
  it('creates, scoped to the organization, toasts, and invalidates the projects root', async () => {
    const { repository, create } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateProject(), { wrapper: Wrapper });

    await result.current.mutateAsync({ name: 'new-project', organizationId: ORG_ID, description: 'd' });

    expect(create).toHaveBeenCalledWith('new-project', ORG_ID, 'd');
    expect(toastSuccess).toHaveBeenCalledWith('Project created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['projects'] });
  });
});

describe('useUpdateProject', () => {
  it('updates and invalidates the projects root', async () => {
    const { repository, update } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateProject(), { wrapper: Wrapper });

    await result.current.mutateAsync({ projectId: 'project-1', name: 'renamed' });

    expect(update).toHaveBeenCalledWith('project-1', 'renamed', undefined);
    expect(toastSuccess).toHaveBeenCalledWith('Project updated');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['projects'] });
  });
});

describe('useDeleteProject', () => {
  it('deletes and invalidates the projects root', async () => {
    const { repository, delete: del } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteProject(), { wrapper: Wrapper });

    await result.current.mutateAsync('project-1');

    expect(del).toHaveBeenCalledWith('project-1');
    expect(toastSuccess).toHaveBeenCalledWith('Project deleted');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['projects'] });
  });
});

describe('useProjects', () => {
  it('fetches the first page for the given organization', async () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useProjects(ORG_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.projects).toHaveLength(1));
    expect(getByOrganizationId).toHaveBeenCalledWith(ORG_ID, { page: 1, pageSize: 10 });
  });

  it('does not fetch when organizationId is null', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useProjects(null), { wrapper: Wrapper });
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });

  it('resets to page 1 when the organization changes', async () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result, rerender } = renderHook(({ orgId }: { orgId: string }) => useProjects(orgId), {
      wrapper: Wrapper,
      initialProps: { orgId: 'org-1' },
    });

    await waitFor(() => expect(result.current.projects).toHaveLength(1));
    act(() => result.current.setPage(2));
    await waitFor(() => expect(result.current.page).toBe(2));

    rerender({ orgId: 'org-2' });

    await waitFor(() => expect(result.current.page).toBe(1));
    expect(getByOrganizationId).toHaveBeenLastCalledWith('org-2', { page: 1, pageSize: 10 });
  });
});

describe('useOrganizationProjects', () => {
  it('fetches the whole list (lookup page) for the organization', async () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizationProjects(ORG_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.projects).toHaveLength(1));
    expect(getByOrganizationId).toHaveBeenCalledWith(ORG_ID, PROJECTS_LOOKUP_PAGE);
  });

  it('does not fetch for a null organizationId', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationProjects(null), { wrapper: Wrapper });
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });

  it('shares its cache entry with useProjectsByOrganizations for the same org (same query key)', async () => {
    const getByOrganizationId = vi
      .fn()
      .mockResolvedValue(ScyllaResult.success({ projects: [project()], pagination: pagination() }));
    const { repository } = makeFakeRepository({ getByOrganizationId });
    const { Wrapper, queryClient } = wrapperFor(repository);

    // Prime the cache the way useOrganizationProjects would.
    renderHook(() => useOrganizationProjects(ORG_ID), { wrapper: Wrapper });
    await waitFor(() =>
      expect(
        queryClient.getQueryData(PROJECTS_QUERY_KEY(ORG_ID, PROJECTS_LOOKUP_PAGE)),
      ).toBeDefined(),
    );

    getByOrganizationId.mockClear();
    const { result } = renderHook(() => useProjectsByOrganizations([ORG_ID]), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.get('project-1')).toBeDefined());
    // Served from the cache primed above - no second network call.
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });
});

describe('useProjectsByOrganizations', () => {
  it('fans out one query per organization and combines into a projectId -> {name, organizationId} map', async () => {
    const getByOrganizationId = vi.fn((organizationId: string) =>
      Promise.resolve(
        ScyllaResult.success({
          projects: [project({ id: `p-${organizationId}`, name: `Project of ${organizationId}` })],
          pagination: pagination(),
        }),
      ),
    );
    const { repository } = makeFakeRepository({ getByOrganizationId });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useProjectsByOrganizations(['org-a', 'org-b']), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.size).toBe(2));
    expect(result.current.get('p-org-a')).toEqual({ name: 'Project of org-a', organizationId: 'org-a' });
    expect(result.current.get('p-org-b')).toEqual({ name: 'Project of org-b', organizationId: 'org-b' });
  });

  it('fetches nothing and returns an empty map when disabled', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useProjectsByOrganizations(['org-a'], false), {
      wrapper: Wrapper,
    });

    expect(getByOrganizationId).not.toHaveBeenCalled();
    expect(result.current.size).toBe(0);
  });

  it('fetches nothing for an empty organizationIds list', () => {
    const { repository, getByOrganizationId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useProjectsByOrganizations([]), { wrapper: Wrapper });
    expect(getByOrganizationId).not.toHaveBeenCalled();
  });
});

describe('useProjectMembers', () => {
  it('lists members for the given project', async () => {
    const { repository, listMembers } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useProjectMembers('project-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.members).toHaveLength(1));
    expect(listMembers).toHaveBeenCalledWith('project-1');
  });

  it('does not fetch for a null projectId', () => {
    const { repository, listMembers } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useProjectMembers(null), { wrapper: Wrapper });
    expect(listMembers).not.toHaveBeenCalled();
  });

  it('refetchMembers invalidates exactly this project\'s members key', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useProjectMembers('project-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.members).toHaveLength(1));
    result.current.refetchMembers();

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: PROJECT_MEMBERS_QUERY_KEY('project-1'),
      exact: true,
    });
  });
});
