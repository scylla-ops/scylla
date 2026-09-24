import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { waitFor } from '@testing-library/svelte';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobEntity } from '@/modules/features/jobs';
import type { PipelineMetadata } from '@/modules/features/pipeline';
import type { ProjectEntity } from '@/modules/features/project';
import { createOrgOverview } from './org-overview.state.svelte.ts';

const project = (overrides: Partial<ProjectEntity> = {}): ProjectEntity =>
  ({ id: 'project-1', name: 'Acme project', ...overrides });

const pipeline = (overrides: Partial<PipelineMetadata> = {}): PipelineMetadata => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci',
  nodeCount: 1,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const job = (overrides: Partial<JobEntity> = {}): JobEntity => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  ...overrides,
});

const page = <T>(items: T[], totalCount = items.length) => ({
  items,
  pagination: { totalCount, page: 1, pageSize: 100, totalPages: 1, hasNext: false, hasPrevious: false },
});

let getProjects: ReturnType<typeof vi.fn>;
let getPipelines: ReturnType<typeof vi.fn>;
let getJobs: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

beforeEach(() => {
  getProjects = vi.fn().mockResolvedValue(ScyllaResult.success({ projects: [project()] }));
  getPipelines = vi.fn().mockResolvedValue(ScyllaResult.success(page([pipeline()])));
  getJobs = vi.fn().mockResolvedValue(ScyllaResult.success(page([job()])));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    project: { projectRepository: { getByOrganizationId: getProjects } },
    pipeline: { pipelineRepository: { getMetadataByOrganizationId: getPipelines } },
    jobs: { jobsRepository: { getByOrganizationId: getJobs } },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  permissionsStore.setState({ permissions: null });
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

/** Builds the ViewModel inside a reactive root — it is made of runes. */
const withOverview = async (
  body: (overview: ReturnType<typeof createOrgOverview>) => Promise<void> | void,
) => {
  let overview!: ReturnType<typeof createOrgOverview>;
  const cleanup = $effect.root(() => {
    overview = createOrgOverview();
  });

  try {
    await body(overview);
  } finally {
    cleanup();
  }
};

describe('createOrgOverview', () => {
  it('reads the current organization from context and asks each module for its own half', async () => {
    await withOverview(async overview => {
      expect(overview.organizationId).toBe('org-1');

      await waitFor(() => expect(overview.projects).toHaveLength(1));
      expect(getProjects).toHaveBeenCalledWith('org-1', { page: 1, pageSize: 100 });
      expect(getPipelines).toHaveBeenCalledWith('org-1', { page: 1, pageSize: 100 });
      expect(getJobs).toHaveBeenCalledWith('org-1', { page: 1, pageSize: 100 });
    });
  });

  it('joins each pipeline with the name of the project it belongs to', async () => {
    await withOverview(async overview => {
      await waitFor(() => expect(overview.allPipelines).toHaveLength(1));
      expect(overview.allPipelines[0].projectName).toBe('Acme project');
    });
  });

  it("leaves projectName empty when the pipeline's project is not in the visible list", async () => {
    getPipelines.mockResolvedValue(
      ScyllaResult.success(page([pipeline({ projectId: 'project-404' })])),
    );

    await withOverview(async overview => {
      await waitFor(() => expect(overview.allPipelines).toHaveLength(1));
      expect(overview.allPipelines[0].projectName).toBe('');
    });
  });

  it('reports the runs window as partial when the server holds more than it returned', async () => {
    getJobs.mockResolvedValue(ScyllaResult.success(page([job()], 250)));

    await withOverview(async overview => {
      await waitFor(() => expect(overview.runsTruncated).toBe(true));
      expect(overview.totalRuns).toBe(250);
    });
  });

  it('reports the runs window as whole when it holds every run there is', async () => {
    await withOverview(async overview => {
      await waitFor(() => expect(overview.recentJobs).toHaveLength(1));
      expect(overview.runsTruncated).toBe(false);
    });
  });

  it('summarizes the recent-runs window', async () => {
    await withOverview(async overview => {
      await waitFor(() => expect(overview.runs.completed).toBe(1));
      expect(overview.runs.total).toBe(1);
    });
  });

  it('surfaces the projects failure, since it is the one the page refuses to render without', async () => {
    getProjects.mockResolvedValue(ScyllaResult.error(new ScyllaError('boom')));

    await withOverview(async overview => {
      await waitFor(() => expect(overview.projectsError).toBe(true));
    });
  });

  describe('canOpenProject', () => {
    it('denies while permissions are unknown', async () => {
      await withOverview(overview => {
        expect(overview.canOpenProject('project-1')).toBe(false);
      });
    });

    it('delegates to LIST_PIPELINES_BY_PROJECT, scoped to the given project id', async () => {
      permissionsStore.setState({
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

      await withOverview(overview => {
        expect(overview.canOpenProject('project-1')).toBe(true);
        expect(overview.canOpenProject('project-2')).toBe(false);
      });
    });
  });
});
