import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { GrpcPipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/data/remote/grpc-pipeline-remote.data-source.ts';
import { DefaultPipelineRepository } from '@/modules/features/pipeline/infrastructure/repository/default-pipeline.repository.ts';
import { grpcTransport } from '@platform/grpc/index.ts';

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
  routes: [
    {
      mount: 'project',
      index: true,
      permission: Permission.LIST_PIPELINES_BY_PROJECT,
      lazy: async () => ({
        Component: (await import('./presentation/ui/dashboard/DashboardPipeline.page.tsx'))
          .DashboardPipelinePage,
      }),
    },
    {
      mount: 'project',
      path: 'create',
      permission: Permission.CREATE_PIPELINE,
      breadcrumb: () => ({ label: msg`Create` }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/editor/PipelineCreation.page.tsx'))
          .PipelineCreationPage,
      }),
    },
    {
      mount: 'project',
      path: 'edit/:pipelineId',
      permission: Permission.UPDATE_PIPELINE,
      breadcrumb: ({ pipelineName }) => ({
        label: msg`Pipeline`,
        highlight: pipelineName,
        detail: msg`Edit`,
      }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/editor/PipelineUpdate.page.tsx'))
          .PipelineUpdatePage,
      }),
    },
    {
      // Owned here rather than by `jobs`: the page needs a Run action, which is
      // a pipeline operation. See PipelineJobsRoute.
      mount: 'project',
      path: 'pipelines/:pipelineId/jobs',
      permission: Permission.LIST_JOBS_BY_PIPELINE,
      breadcrumb: ({ pipelineName }) => ({
        label: msg`Pipeline`,
        highlight: pipelineName,
        detail: msg`Jobs`,
      }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/PipelineJobsRoute.tsx')).PipelineJobsRoute,
      }),
    },
  ],
} satisfies ScyllaModule;
