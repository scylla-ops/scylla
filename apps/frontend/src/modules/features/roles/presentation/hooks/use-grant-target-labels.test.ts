import { describe, it, expect, vi } from 'vitest';
import { renderHookWithProviders } from '@/test/render.tsx';
import { stubQuery } from '@/test/queries.ts';
import { PermissionScope } from '@platform/authz';
import { useGrantTargetLabels } from './use-grant-target-labels';
import type { ProjectLookupEntry } from '@/modules/features/project';

let organizationsFixture: { id: string; name: string }[] = [];
vi.mock('@/modules/features/organization', () => ({
  organizationQueries: { mine: () => stubQuery(['organizations', 'mine'], organizationsFixture) },
}));

let projectInfoFixture = new Map<string, ProjectLookupEntry>();
vi.mock('@/modules/features/project', () => ({
  // `useQueries` is handed the queries *and* the combine; stubbing the combine
  // is what lets the fixture be a finished map rather than per-organization
  // pages.
  projectLookupQueries: () => ({ queries: [], combine: () => projectInfoFixture }),
}));

describe('useGrantTargetLabels', () => {
  it('SYSTEM scope always labels as "System", regardless of scopeId', () => {
    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.SYSTEM));
    expect(result.current.labelFor('anything')).toEqual({ name: 'System', resolved: true });
  });

  it('an empty scopeId also labels as "System" (the SYSTEM-scope convention)', () => {
    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.ORGANIZATION));
    expect(result.current.labelFor('')).toEqual({ name: 'System', resolved: true });
  });

  it('ORGANIZATION scope resolves the id to the organization name once loaded', () => {
    organizationsFixture = [{ id: 'org-1', name: 'Scylla Inc' }];
    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.ORGANIZATION));
    expect(result.current.labelFor('org-1')).toEqual({ name: 'Scylla Inc', resolved: true });
  });

  it('ORGANIZATION scope falls back to the raw id, unresolved, before the name has loaded', () => {
    organizationsFixture = [];
    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.ORGANIZATION));
    expect(result.current.labelFor('org-unknown')).toEqual({ name: 'org-unknown', resolved: false });
  });

  it('PROJECT scope resolves both the project name and its owning organization\'s name', () => {
    organizationsFixture = [{ id: 'org-1', name: 'Scylla Inc' }];
    projectInfoFixture = new Map([['project-1', { name: 'ci-platform', organizationId: 'org-1' }]]);

    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.PROJECT));
    expect(result.current.labelFor('project-1')).toEqual({
      name: 'ci-platform',
      organizationName: 'Scylla Inc',
      resolved: true,
    });
  });

  it('PROJECT scope falls back to the raw id, unresolved, before the project lookup has loaded', () => {
    projectInfoFixture = new Map();
    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.PROJECT));
    expect(result.current.labelFor('project-unknown')).toEqual({
      name: 'project-unknown',
      resolved: false,
    });
  });

  it('PROJECT scope falls back to the raw organization id when the org name itself is not yet loaded', () => {
    organizationsFixture = [];
    projectInfoFixture = new Map([['project-1', { name: 'ci-platform', organizationId: 'org-1' }]]);

    const { result } = renderHookWithProviders(() => useGrantTargetLabels(PermissionScope.PROJECT));
    expect(result.current.labelFor('project-1')).toEqual({
      name: 'ci-platform',
      organizationName: 'org-1',
      resolved: true,
    });
  });
});
