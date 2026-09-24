import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { GrpcPipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/data/remote/grpc-pipeline-remote.data-source.ts';
import { DefaultPipelineRepository } from '@/modules/features/pipeline/infrastructure/repository/default-pipeline.repository.ts';
import { grpcTransport } from '@platform/grpc';

const pipelineRemoteDataSource: PipelineRemoteDataSource = new GrpcPipelineRemoteDataSource(
  grpcTransport,
);
const pipelineRepository = new DefaultPipelineRepository(pipelineRemoteDataSource);

export const PipelineModule = {
  id: 'pipeline',
  domain: {
    /** Repository interface — the module's data surface. */
    pipelineRepository: pipelineRepository,
  },
  routes: {
    project: [
      {
        permission: Permission.LIST_PIPELINES_BY_PROJECT,
        page: () => import('./presentation/ui/dashboard/DashboardPipeline/DashboardPipeline.page.svelte'),
      },
      {
        path: 'create',
        permission: Permission.CREATE_PIPELINE,
        breadcrumb: () => ({ label: msg`Create` }),
        page: () => import('./presentation/ui/editor/PipelineCreation.page.svelte'),
      },
      {
        path: 'edit/:pipelineId',
        permission: Permission.UPDATE_PIPELINE,
        breadcrumb: ({ pipelineName }) => ({
          label: msg`Pipeline`,
          highlight: pipelineName,
          detail: msg`Edit`,
        }),
        page: () => import('./presentation/ui/editor/PipelineUpdate.page.svelte'),
      },
      {
        // Owned here rather than by `jobs`: the page needs a Run action, which is
        // a pipeline operation. One job's page, from `jobs`, shows this crumb too.
        path: 'pipelines/:pipelineId/jobs',
        permission: Permission.LIST_JOBS_BY_PIPELINE,
        breadcrumb: ({ pipelineName }) => ({
          label: msg`Pipeline`,
          highlight: pipelineName,
          detail: msg`Jobs`,
        }),
        page: () => import('./presentation/ui/PipelineJobsRoute.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
