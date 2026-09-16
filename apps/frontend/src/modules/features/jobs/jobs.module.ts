import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import { grpcTransport } from '@platform/grpc';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import { GrpcJobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/data/remote/grpc-jobs-remote.data-source.ts';
import { DefaultJobsRepository } from '@/modules/features/jobs/infrastructure/repository/default-jobs.repository.ts';

const jobsRemoteDataSource: JobsRemoteDataSource = new GrpcJobsRemoteDataSource(
  grpcTransport,
);
const jobsRepository = new DefaultJobsRepository(jobsRemoteDataSource);

export const JobsModule = {
  id: 'jobs',
  domain: {
    /** Repository interface — the module's data surface. */
    jobsRepository: jobsRepository,
  },
  routes: [
    {
      // The jobs *list* is mounted by `pipeline` (it owns the Run action); one
      // job needs nothing from that module, so it is declared here — which also
      // keeps the page out of this module's public API.
      mount: 'project',
      path: 'pipelines/:pipelineId/jobs/:jobId',
      permission: Permission.READ_JOB,
      breadcrumb: ({ pipelineName }) => ({
        label: msg`Pipeline`,
        highlight: pipelineName,
        detail: msg`Job`,
      }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/JobDetails.page.tsx')).JobDetailsPage,
      }),
    },
  ],
} satisfies ScyllaModule;
