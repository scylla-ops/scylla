import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope, usePermissionsStore } from '@platform/authz';
import { setAppNavigator, useContextStore } from '@platform/context';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import DashboardPage from './Dashboard.page.svelte';

const project = (overrides: Record<string, unknown> = {}) => ({
  id: 'project-1',
  name: 'Acme project',
  description: 'The main one',
  ...overrides,
});

const pipeline = (overrides: Record<string, unknown> = {}) => ({
  id: 'pipeline-1',
  projectId: 'project-1',
  name: 'ci',
  nodeCount: 3,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const job = (overrides: Record<string, unknown> = {}) => ({
  id: 'job-1',
  pipelineId: 'pipeline-1',
  status: 'completed',
  nodeExecutions: [],
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:01:00.000Z',
  ...overrides,
});

const page = (items: unknown[], totalCount = items.length) => ({
  items,
  pagination: { totalCount, page: 1, pageSize: 100, totalPages: 1, hasNext: false, hasPrevious: false },
});

let getProjects: ReturnType<typeof vi.fn>;
let getPipelines: ReturnType<typeof vi.fn>;
let getJobs: ReturnType<typeof vi.fn>;
let listAgents: ReturnType<typeof vi.fn>;
let navigate: ReturnType<typeof vi.fn<(to: string, options?: unknown) => void>>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

/** A system-wide grant of exactly these permissions. */
const grant = (permissions: Permission[]) =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access: { kind: 'restricted', permissions },
        },
      ],
    },
  });

beforeEach(() => {
  getProjects = vi.fn().mockResolvedValue(ScyllaResult.success({ projects: [project()] }));
  getPipelines = vi.fn().mockResolvedValue(ScyllaResult.success(page([pipeline()])));
  getJobs = vi.fn().mockResolvedValue(ScyllaResult.success(page([job()])));
  listAgents = vi.fn().mockResolvedValue(ScyllaResult.success([]));
  navigate = vi.fn();

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    project: { projectRepository: { getByOrganizationId: getProjects } },
    pipeline: { pipelineRepository: { getMetadataByOrganizationId: getPipelines } },
    jobs: { jobsRepository: { getByOrganizationId: getJobs } },
    agents: { agentsRepository: { listAgents, getAgentStats: vi.fn() } },
  });

  setAppNavigator({ navigate, back: vi.fn(), pathname: () => '/', search: () => '' });
  useContextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  } as never);
  grant([Permission.LIST_PIPELINES_BY_PROJECT]);
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  setAppNavigator(null);
  usePermissionsStore.setState({ permissions: null });
});

/**
 * The card a piece of text sits in.
 *
 * Two things on this page legitimately carry the same string — a project's card
 * title and the badge naming that project in the pipeline row — and one figure
 * only means something next to its own label. `data-slot` is ours, not a
 * library's, so it is a fair anchor; the table rows are not cards, which is what
 * makes this unambiguous.
 */
const cardContaining = async (text: string): Promise<HTMLElement> => {
  const matches = await screen.findAllByText(text);
  const card = matches.map(node => node.closest('[data-slot="card"]')).find(Boolean);
  if (!card) throw new Error(`No card contains "${text}"`);
  return card as HTMLElement;
};

describe('DashboardPage', () => {
  it('counts projects, pipelines and runs from the three modules that own them', async () => {
    render(DashboardPage);

    expect(await screen.findByText('ci')).toBeInTheDocument();
    // One project, one pipeline, one run — each figure inside its own tile.
    for (const label of ['Projects', 'Pipelines', 'Runs']) {
      expect(within(await cardContaining(label)).getByText('1')).toBeInTheDocument();
    }
  });

  it('shows an em dash rather than 0% when nothing has finished yet', async () => {
    getJobs.mockResolvedValue(ScyllaResult.success(page([job({ status: 'running' })])));
    render(DashboardPage);

    expect(await screen.findByText('—')).toBeInTheDocument();
  });

  it('says the success rate is recent when the window does not cover every run', async () => {
    getJobs.mockResolvedValue(ScyllaResult.success(page([job()], 250)));
    render(DashboardPage);

    expect(await screen.findByText('Success rate (recent)')).toBeInTheDocument();
  });

  it('opens a project the user may enter', async () => {
    const user = userEvent.setup();
    grant([]);
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
    render(DashboardPage);

    await user.click(await cardContaining('Acme project'));

    expect(navigate).toHaveBeenCalled();
  });

  it('leaves a project the user may see but not open inert, and says why', async () => {
    const user = userEvent.setup();
    grant([]);
    render(DashboardPage);

    const card = await cardContaining('Acme project');
    expect(card).toHaveAttribute('title', "You don't have access to this project's pipelines");

    await user.click(card);
    expect(navigate).not.toHaveBeenCalled();
  });

  it('joins each pipeline row with the name of its project', async () => {
    render(DashboardPage);

    const row = (await screen.findByText('ci')).closest('tr');
    expect(within(row!).getByText('Acme project')).toBeInTheDocument();
    expect(within(row!).getByText('3')).toBeInTheDocument();
  });

  it('hides the agent chart whole from a user without READ_APP_STATS', async () => {
    render(DashboardPage);

    expect(await screen.findByText('ci')).toBeInTheDocument();
    expect(screen.queryByText('Agent Outcomes')).not.toBeInTheDocument();
  });

  it('shows the agent chart once READ_APP_STATS is held', async () => {
    grant([Permission.READ_APP_STATS, Permission.LIST_AGENTS]);
    render(DashboardPage);

    expect(await screen.findByText('Agent Outcomes')).toBeInTheDocument();
  });

  it('replaces the whole page with an error state when the project list fails', async () => {
    getProjects.mockResolvedValue(ScyllaResult.error(new ScyllaError('boom')));
    render(DashboardPage);

    expect(await screen.findByText('Unable to load dashboard')).toBeInTheDocument();
    expect(screen.queryByText('All Pipelines')).not.toBeInTheDocument();
  });

  it('sends "See all" to the organization project list', async () => {
    const user = userEvent.setup();
    render(DashboardPage);

    await user.click(await screen.findByText('See all'));

    expect(navigate).toHaveBeenCalledWith(expect.stringContaining('/projects'), undefined);
  });
});
