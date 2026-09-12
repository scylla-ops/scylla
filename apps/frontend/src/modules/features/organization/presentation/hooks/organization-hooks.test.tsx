import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { useContextStore } from '@platform/context';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useCreateOrganization } from './useCreateOrganization';
import { useDeleteOrganization } from './use-delete-organization';
import { useOrganizations } from './useOrganizations';
import { useUpdateOrganization } from './use-update-organization';
import { useOrganizationMembers, ORGANIZATION_MEMBERS_QUERY_KEY } from './use-organization-members';
import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { OrganizationEntity } from '@/modules/features/organization/domain/entities/organization.entity.ts';
import type { UserEntity } from '@/modules/features/user';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const ORG_ID = 'org-1';

const org = (overrides: Partial<OrganizationEntity> = {}): OrganizationEntity => ({
  id: ORG_ID,
  name: 'Scylla Inc',
  description: 'a test org',
  ...overrides,
});

const member = (overrides: Partial<UserEntity> = {}): UserEntity => ({
  userId: 'user-1',
  username: 'ravenne',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<OrganizationRepository> = {}) => {
  // Each mock falls back to overrides[method] first, so the const returned
  // below always matches what's actually on `repository` (an override passed
  // in isn't shadowed by a second, unused default mock — see the fixed bug in
  // the earlier triggers-hooks batch).
  const getAll = overrides.getAll ?? vi.fn().mockResolvedValue(ScyllaResult.success([org()]));
  const getMine = overrides.getMine ?? vi.fn().mockResolvedValue(ScyllaResult.success([org()]));
  const listMembers =
    overrides.listMembers ?? vi.fn().mockResolvedValue(ScyllaResult.success([member()]));
  const create = overrides.create ?? vi.fn().mockResolvedValue(ScyllaResult.success(org()));
  const update = overrides.update ?? vi.fn().mockResolvedValue(ScyllaResult.success(org()));
  const del = overrides.delete ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  const repository: OrganizationRepository = {
    getAll,
    getMine,
    listMembers,
    create,
    update,
    delete: del,
  };
  return { repository, getAll, getMine, listMembers, create, update, delete: del };
};

const wrapperFor = (repository: OrganizationRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ organization: { organizationRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  useContextStore.getState().reset();
});

describe('useOrganizations', () => {
  it('lists the organizations the current user belongs to (getMine, not getAll)', async () => {
    const { repository, getMine, getAll } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizations(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.organizations).toHaveLength(1));
    expect(getMine).toHaveBeenCalled();
    expect(getAll).not.toHaveBeenCalled();
  });

  it('surfaces an error as isError', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      getMine: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizations(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

describe('useCreateOrganization', () => {
  it('creates an organization, switches the active context to it, toasts, and invalidates the list', async () => {
    const created = org({ id: 'org-new', name: 'New Org' });
    const { repository, create } = makeFakeRepository({
      create: vi.fn().mockResolvedValue(ScyllaResult.success(created)),
    });
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateOrganization(), { wrapper: Wrapper });

    await result.current.mutateAsync({ name: 'New Org', description: 'd' });

    expect(create).toHaveBeenCalledWith('New Org', 'd');
    expect(useContextStore.getState().organization).toEqual({ id: 'org-new', name: 'New Org' });
    expect(toastSuccess).toHaveBeenCalledWith('Organization created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['organizations'] });
  });
});

describe('useUpdateOrganization', () => {
  it('updates and invalidates the organizations list', async () => {
    const { repository, update } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateOrganization(), { wrapper: Wrapper });

    await result.current.mutateAsync({ organizationId: ORG_ID, name: 'Renamed' });

    expect(update).toHaveBeenCalledWith(ORG_ID, 'Renamed', undefined);
    expect(toastSuccess).toHaveBeenCalledWith('Organization updated');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['organizations'] });
  });
});

describe('useDeleteOrganization', () => {
  it('deletes and invalidates the organizations list', async () => {
    const { repository, delete: del } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteOrganization(), { wrapper: Wrapper });

    await result.current.mutateAsync(ORG_ID);

    expect(del).toHaveBeenCalledWith(ORG_ID);
    expect(toastSuccess).toHaveBeenCalledWith('Organization deleted');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['organizations'] });
  });
});

describe('useOrganizationMembers', () => {
  it('lists members for the given organization', async () => {
    const { repository, listMembers } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useOrganizationMembers(ORG_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.members).toHaveLength(1));
    expect(listMembers).toHaveBeenCalledWith(ORG_ID);
  });

  it('does not fetch when organizationId is null', () => {
    const { repository, listMembers } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationMembers(null), { wrapper: Wrapper });
    expect(listMembers).not.toHaveBeenCalled();
  });

  it('does not fetch when explicitly disabled via options, even with a valid id', () => {
    const { repository, listMembers } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useOrganizationMembers(ORG_ID, { enabled: false }), { wrapper: Wrapper });
    expect(listMembers).not.toHaveBeenCalled();
  });

  it('refetchMembers invalidates exactly this organization\'s members key', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useOrganizationMembers(ORG_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.members).toHaveLength(1));
    result.current.refetchMembers();

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(ORG_ID),
      exact: true,
    });
  });

  it('refetchMembers is a no-op when there is no organizationId', () => {
    const { repository } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useOrganizationMembers(null), { wrapper: Wrapper });

    result.current.refetchMembers();
    expect(invalidateSpy).not.toHaveBeenCalled();
  });
});
