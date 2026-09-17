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
      // Declared as a child of the jobs path `pipeline` owns — the list needs a
      // Run action, one job does not — so the trail keeps "Jobs" as a crumb of
      // its own. `mergeSharedParents` folds the two declarations of this segment
      // into one route; repeating the path literally is what pairs them, and
      // neither module imports the other.
      mount: 'project',
      path: 'pipelines/:pipelineId/jobs',
      children: [
        {
          mount: 'project',
          path: ':jobId',
          permission: Permission.READ_JOB,
          breadcrumb: ({ jobId }) => ({ label: msg`Job`, highlight: jobId }),
          lazy: async () => ({
            Component: (await import('./presentation/ui/JobDetails.page.tsx')).JobDetailsPage,
          }),
        },
      ],
    },
  ],
} satisfies ScyllaModule;
