import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { flushSync } from 'svelte';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { createRunPipeline } from '../run-pipeline.svelte.ts';

const toast = vi.hoisted(() => ({ success: vi.fn(), warning: vi.fn(), error: vi.fn() }));
vi.mock('svelte-sonner', () => ({ toast }));

let teardown: Array<() => void> = [];
let runRepository: ReturnType<typeof vi.fn>;

const grant = (...permissions: Permission[]) =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

const setUp = (agents: { connected: boolean }[]) => {
  runRepository = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const cache = withQueryClient();
  teardown = [
    cache.restore,
    withRegistry({
      pipeline: { pipelineRepository: { run: runRepository } },
      agents: {
        agentsRepository: {
          listAgents: vi.fn().mockResolvedValue(ScyllaResult.success(agents)),
        },
      },
    }),
  ];
};

const runIn = async (run: (runner: ReturnType<typeof createRunPipeline>) => Promise<void>) => {
  let runner!: ReturnType<typeof createRunPipeline>;
  const cleanup = $effect.root(() => {
    runner = createRunPipeline();
  });
  flushSync();
  await run(runner);
  cleanup();
};

beforeEach(() => {
  vi.clearAllMocks();
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme' } });
});

afterEach(() => teardown.forEach(restore => restore()));

describe('createRunPipeline', () => {
  it('runs the pipeline and tells the user to check the agents when it cannot list them', async () => {
    grant();
    setUp([]);

    await runIn(async runner => {
      await runner.run('pipeline-1');
    });

    expect(runRepository).toHaveBeenCalledWith('pipeline-1');
    expect(toast.success).toHaveBeenCalledTimes(1);
  });

  it('marks the pipeline as running while the run is in flight', async () => {
    grant();
    setUp([]);

    await runIn(async runner => {
      const pending = runner.run('pipeline-1');
      expect(runner.isRunning('pipeline-1')).toBe(true);
      await pending;
      expect(runner.isRunning('pipeline-1')).toBe(false);
    });
  });

  it('keeps quiet about a failed run: the global handler reports it', async () => {
    grant();
    setUp([]);
    runRepository.mockResolvedValue(ScyllaResult.error(new ScyllaError('down')));

    await runIn(async runner => {
      await runner.run('pipeline-1');
      expect(runner.isRunning('pipeline-1')).toBe(false);
    });

    expect(toast.success).not.toHaveBeenCalled();
    expect(toast.warning).not.toHaveBeenCalled();
  });
});

describe('createRunPipeline with the agent list', () => {
  it('warns that the job waits when no agent is connected', async () => {
    grant(Permission.LIST_AGENTS);
    setUp([{ connected: false }]);

    await runIn(async runner => {
      await new Promise(resolve => setTimeout(resolve, 0));
      await runner.run('pipeline-1');
    });

    expect(toast.warning).toHaveBeenCalledTimes(1);
  });

  it('confirms the run when an agent is connected', async () => {
    grant(Permission.LIST_AGENTS);
    setUp([{ connected: true }]);

    await runIn(async runner => {
      await new Promise(resolve => setTimeout(resolve, 0));
      await runner.run('pipeline-1');
    });

    expect(toast.success).toHaveBeenCalledTimes(1);
    expect(toast.warning).not.toHaveBeenCalled();
  });
});
