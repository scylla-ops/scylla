import { i18n } from '@lingui/core';
import { getModuleDomain } from '@platform/di';
import { scyllaNavigate, contextStore } from '@platform/context';
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

// Resolved per call: tests swap the registry.
const repository = () =>
  getModuleDomain<typeof PipelineModule.domain>('pipeline').pipelineRepository;

const currentProject = () => contextStore.getState().project;

export const pipelineQueries = {
  /** `enabled`: wait until the page size is measured, or the first page is fetched twice. */
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

  /** One call, scoped by the backend: nothing to gate here. */
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

  byId: (pipelineId: string) =>
    queryOptions<PipelineEntity>({
      queryKey: PIPELINE_QUERY_KEY(pipelineId),
      enabled: !!pipelineId,
      queryFn: async () => (await repository().getById(pipelineId)).unwrap(),
      staleTime: 30_000,
    }),
};

/** The single page, and whether it covers the whole list. */
export const asPipelineFeed = (data: PaginatedList<PipelineMetadata> | undefined) => {
  const pipelines = data?.items ?? [];
  const totalCount = data?.pagination.totalCount ?? 0;

  return {
    pipelines,
    totalCount,
    isPartialWindow: totalCount > pipelines.length,
  };
};

export interface EditPipelineInput {
  id: string;
  nodes: PipelineStep[];
  name?: string;
}

/** Needs the name too: `goToProject` writes it into the context (the breadcrumb). */
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

  /** The backend has no duplicate: read the pipeline, then create a copy of its steps. */
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

  /** No toast here: `run-pipeline.svelte.ts` decides it from the agents. */
  run: () =>
    mutationOptions({
      mutationFn: async (pipelineId: string) => (await repository().run(pipelineId)).unwrap(),
      onSuccess: (_data, pipelineId) => {
        void getQueryClient().invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId) });
      },
    }),
};
