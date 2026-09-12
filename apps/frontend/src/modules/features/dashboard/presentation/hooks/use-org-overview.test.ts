import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useContextStore } from '@platform/context';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';
import { useOrgOverview } from './use-org-overview';
import type * as ProjectModule from '@/modules/features/project';
import type * as PipelineModule from '@/modules/features/pipeline';
import type * as JobsModule from '@/modules/features/jobs';
import { summarizeJobs } from '@/modules/features/jobs';

type ProjectEntity = ProjectModule.ProjectEntity;
type PipelineMetadata = PipelineModule.PipelineMetadata;
type JobEntity = JobsModule.JobEntity;

let projectsFixture: ProjectEntity[] = [];
let projectsLoadingFixture = false;
let projectsErrorFixture = false;

let pipelinesFixture: PipelineMetadata[] = [];
let pipelinesLoadingFixture = false;
let pipelinesTruncatedFixture = false;

let jobsFixture: JobEntity[] = [];
let jobsLoadingFixture = false;
let jobsTruncatedFixture = false;
let jobsTotalFixture = 0;

vi.mock('@/modules/features/project', async importOriginal => {
  const actual = await importOriginal<typeof ProjectModule>();
  return {
    ...actual,
    useOrganizationProjects: () => ({
      projects: projectsFixture,
      isLoading: projectsLoadingFixture,
      isError: projectsErrorFixture,
    }),
  };
});

vi.mock('@/modules/features/pipeline', async importOriginal => {
  const actual = await importOriginal<typeof PipelineModule>();
  return {
    ...actual,
    useOrganizationPipelines: () => ({
      pipelines: pipelinesFixture,
      isLoading: pipelinesLoadingFixture,
      isPartialWindow: pipelinesTruncatedFixture,
    }),
  };
});

vi.mock('@/modules/features/jobs', async importOriginal => {
  const actual = await importOriginal<typeof JobsModule>();
  return {
    ...actual,
    useOrganizationJobs: () => ({
      jobs: jobsFixture,
      summary: summarizeJobs(jobsFixture),
      totalCount: jobsTotalFixture,
      isLoading: jobsLoadingFixture,
      isPartialWindow: jobsTruncatedFixture,
    }),
  };
});

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity => ({
  id: 'project-1',
  name: 'Acme project',
  ...overrides,
});

const pipeline = (overrides: Partial<PipelineMetadata> = {}): PipelineMetadata => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci',
  nodeCount: 1,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

beforeEach(() => {
  projectsFixture = [];
  projectsLoadingFixture = false;
  projectsErrorFixture = false;
  pipelinesFixture = [];
  pipelinesLoadingFixture = false;
  pipelinesTruncatedFixture = false;
  jobsFixture = [];
  jobsLoadingFixture = false;
  jobsTruncatedFixture = false;
  jobsTotalFixture = 0;

  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
    pipeline: { id: null, name: null },
  });
  usePermissionsStore.setState({ permissions: null });
});

describe('useOrgOverview', () => {
  it('reads the current organizationId from context and passes it straight through', () => {
    const { result } = renderHook(() => useOrgOverview());
    expect(result.current.organizationId).toBe('org-1');
  });

  it('joins each pipeline with the name of the project it belongs to', () => {
    projectsFixture = [project({ id: 'project-1', name: 'Acme project' })];
    pipelinesFixture = [pipeline({ projectId: 'project-1' })];

    const { result } = renderHook(() => useOrgOverview());

    expect(result.current.allPipelines).toEqual([
      { ...pipeline({ projectId: 'project-1' }), projectName: 'Acme project' },
    ]);
  });

  it('joins to an empty projectName when the pipeline\'s project is not in the (visible) project list', () => {
    projectsFixture = [];
    pipelinesFixture = [pipeline({ projectId: 'project-404' })];

    const { result } = renderHook(() => useOrgOverview());

    expect(result.current.allPipelines[0].projectName).toBe('');
  });

  it('passes through each half\'s own loading/error/truncation flags unchanged', () => {
    projectsLoadingFixture = true;
    projectsErrorFixture = true;
    pipelinesLoadingFixture = true;
    pipelinesTruncatedFixture = true;
    jobsLoadingFixture = true;
    jobsTruncatedFixture = true;

    const { result } = renderHook(() => useOrgOverview());

    expect(result.current.projectsLoading).toBe(true);
    expect(result.current.projectsError).toBe(true);
    expect(result.current.pipelinesLoading).toBe(true);
    expect(result.current.pipelinesTruncated).toBe(true);
    expect(result.current.runsLoading).toBe(true);
    expect(result.current.runsTruncated).toBe(true);
  });

  it('summarizes and forwards the recent-runs window as-is', () => {
    jobsFixture = [
      {
        id: 'job-1',
        pipelineId: 'pipeline-1',
        status: 'completed',
        nodeExecutions: [],
        createdAt: '2026-01-01T00:00:00.000Z',
        updatedAt: '2026-01-01T00:01:00.000Z',
      },
    ];
    jobsTotalFixture = 1;

    const { result } = renderHook(() => useOrgOverview());

    expect(result.current.recentJobs).toEqual(jobsFixture);
    expect(result.current.runs.completed).toBe(1);
    expect(result.current.totalRuns).toBe(1);
  });

  describe('canOpenProject', () => {
    it('denies while permissions are unknown', () => {
      const { result } = renderHook(() => useOrgOverview());
      expect(result.current.canOpenProject('project-1')).toBe(false);
    });

    it('delegates to LIST_PIPELINES_BY_PROJECT, scoped to the given project id', () => {
      usePermissionsStore.setState({
        permissions: {
          scopes: [
            {
              scope: PermissionScope.PROJECT,
              scopeId: 'project-1',
              access: { kind: 'restricted', permissions: [Permission.LIST_PIPELINES_BY_PROJECT] },
            },
          ],
        },
      });

      const { result } = renderHook(() => useOrgOverview());
      expect(result.current.canOpenProject('project-1')).toBe(true);
      expect(result.current.canOpenProject('project-2')).toBe(false);
    });
  });
});
