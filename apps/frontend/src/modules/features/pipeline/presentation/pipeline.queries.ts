import { i18n } from '@lingui/core';
import { getModuleDomain } from '@platform/di';
import { scyllaNavigate, useContextStore } from '@platform/context';
import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { JOBS_QUERY_KEY } from '@/modules/features/jobs';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { PipelineEntity } from '../domain/entities/pipeline.entity.ts';
import type { PipelineMetadata, PipelineStep } from '../domain/structs/pipeline.struct.ts';
import type { PipelineModule } from '../pipeline.module.ts';
import { pipelineMessages } from './pipeline.messages.ts';
import {
  ORGANIZATION_PIPELINES_QUERY_KEY,
  PIPELINES_LOOKUP_PAGE,
  PIPELINES_QUERY_KEY,
  PIPELINES_QUERY_ROOT,
  PIPELINE_QUERY_KEY,
  PROJECT_PIPELINES_QUERY_ROOT,
} from './pipelines.query-keys.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what the eight hooks under `presentation/hooks/` were, with the
 * framework taken out. `byOrganization` predates the rest — `dashboard` went
 * Svelte in Phase 4 and reads it through the barrel — and the others joined it
 * here when the module itself was ported.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () =>
  getModuleDomain<typeof PipelineModule.domain>('pipeline').pipelineRepository;

/** The project the user is currently in, which is where a write lands. */
const currentProject = () => useContextStore.getState().project;

export const pipelineQueries = {
  /**
   * One page of a project's pipelines.
   *
   * `enabled` takes the caller's readiness because the page size is measured
   * from the layout: asking before the table area exists would fetch a page
   * sized for a container that has not been laid out yet, then fetch again.
   */
  byProject: (
    projectId: string,
    pagination: PaginationParams,
    options: { enabled?: boolean } = {},
  ) =>
    queryOptions<PaginatedList<PipelineMetadata>>({
      queryKey: PIPELINES_QUERY_KEY(projectId, pagination),
      enabled: (options.enabled ?? true) && !!projectId,
      queryFn: async () =>
        (await repository().getMetadataByProjectId(projectId, pagination)).unwrap(),
      staleTime: 5_000,
    }),

  /**
   * Every pipeline of one organization, in a single request.
   *
   * The replacement for the per-project fan-out the dashboard used to run: that
   * cost one request and one cache entry per project and — because
   * `ListPipelinesByProject` is enforced per project — one error toast for
   * every project the caller could not read. `ListOrganizationPipelines` is
   * scoped server-side, so there is nothing to gate client-side.
   */
  byOrganization: (organizationId: string | null) =>
    queryOptions<PaginatedList<PipelineMetadata>>({
      queryKey: ORGANIZATION_PIPELINES_QUERY_KEY(organizationId, PIPELINES_LOOKUP_PAGE),
      enabled: !!organizationId,
      queryFn: async () =>
        (
          await repository().getMetadataByOrganizationId(organizationId!, PIPELINES_LOOKUP_PAGE)
        ).unwrap(),
      staleTime: 30_000,
    }),

  /** One pipeline, steps included — the editor's source document. */
  byId: (pipelineId: string) =>
    queryOptions<PipelineEntity>({
      queryKey: PIPELINE_QUERY_KEY(pipelineId),
      enabled: !!pipelineId,
      queryFn: async () => (await repository().getById(pipelineId)).unwrap(),
      staleTime: 30_000,
    }),
};

/**
 * The shape both consumers derive from one page of pipeline metadata.
 *
 * A pure fold rather than a second query: the window is a single page, so
 * whether it covers the whole list is something the page itself answers.
 */
export const asPipelineFeed = (data: PaginatedList<PipelineMetadata> | undefined) => {
  const pipelines = data?.items ?? [];
  const totalCount = data?.pagination.totalCount ?? 0;

  return {
    pipelines,
    totalCount,
    /** True when more pipelines exist than the single page fetched. */
    isPartialWindow: totalCount > pipelines.length,
  };
};

export interface EditPipelineInput {
  id: string;
  nodes: PipelineStep[];
  name?: string;
}

/**
 * Back to the project the write belongs to — only when there is one.
 *
 * Both halves are needed, not just the id: `goToProject` also writes the name
 * into the context store, and sending it an empty one would blank the
 * breadcrumb the user lands on.
 */
const returnToProject = () => {
  const { id, name } = currentProject();
  if (id && name) scyllaNavigate.goToProject(id, name);
};

export const pipelineMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async (pipeline: Omit<PipelineEntity, 'id'>) =>
        (await repository().create(pipeline)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.PIPELINE_CREATE));
        const { id, name } = currentProject();
        if (!id || !name) return;

        void getQueryClient().invalidateQueries({ queryKey: PROJECT_PIPELINES_QUERY_ROOT(id) });
        returnToProject();
      },
    }),

  update: () =>
    mutationOptions({
      mutationFn: async ({ id, nodes, name }: EditPipelineInput) =>
        (await repository().edit(id, nodes, name)).unwrap(),
      onSuccess: pipeline => {
        const queryClient = getQueryClient();
        void queryClient.invalidateQueries({
          queryKey: PROJECT_PIPELINES_QUERY_ROOT(pipeline.projectId),
        });
        void queryClient.invalidateQueries({ queryKey: PIPELINE_QUERY_KEY(pipeline.id) });
        toast.success(i18n._(ToastMessages.PIPELINE_UPDATE));
        returnToProject();
      },
    }),

  remove: () =>
    mutationOptions({
      mutationFn: async (pipelineId: string) =>
        (await repository().deleteById(pipelineId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.PIPELINE_DELETE));
        void getQueryClient().invalidateQueries({ queryKey: PIPELINES_QUERY_ROOT });
      },
    }),

  /**
   * Copies a pipeline: read it whole, then create a second one from its steps.
   *
   * Two calls with a rule between them, which is why this one is not a plain
   * forward to a repository method — the backend has no duplicate operation and
   * the "(copy)" name is a decision made here.
   */
  duplicate: () =>
    mutationOptions({
      mutationFn: async (pipelineId: string) => {
        const pipeline = (await repository().getById(pipelineId)).unwrap();

        (
          await repository().create({
            name: i18n._(pipelineMessages.copyOf(pipeline.name)),
            projectId: pipeline.projectId,
            nodes: pipeline.nodes,
          })
        ).unwrap();
      },
      onSuccess: () => {
        void getQueryClient().invalidateQueries({ queryKey: PIPELINES_QUERY_ROOT });
        toast.success(i18n._(ToastMessages.PIPELINE_DUPLICATE));
        returnToProject();
      },
    }),

  /**
   * Starts a run.
   *
   * The toast is not here: whether to reassure, warn about agent connectivity
   * or say nothing depends on the agent list, which only a subscriber holds —
   * see `run-pipeline.svelte.ts`. Invalidating the pipeline's jobs is
   * unconditional, because a run creates one either way.
   */
  run: () =>
    mutationOptions({
      mutationFn: async (pipelineId: string) => (await repository().run(pipelineId)).unwrap(),
      onSuccess: (_data, pipelineId) => {
        void getQueryClient().invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId) });
      },
    }),
};
