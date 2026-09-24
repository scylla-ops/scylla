import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import { grpcTransport } from '@platform/grpc';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import { GrpcJobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/data/remote/grpc-jobs-remote.data-source.ts';
import { DefaultJobsRepository } from '@/modules/features/jobs/infrastructure/repository/default-jobs.repository.ts';

const jobsRemoteDataSource: JobsRemoteDataSource = new GrpcJobsRemoteDataSource(grpcTransport);
const jobsRepository = new DefaultJobsRepository(jobsRemoteDataSource);

export const JobsModule = {
  id: 'jobs',
  domain: {
    jobsRepository: jobsRepository,
  },
  routes: {
    project: [
      {
        // Under the jobs list of `pipeline`, whose crumb shows first.
        path: 'pipelines/:pipelineId/jobs/:jobId',
        permission: Permission.READ_JOB,
        breadcrumb: ({ jobId }) => ({ label: msg`Job`, highlight: jobId }),
        page: () => import('./presentation/ui/JobDetails/JobDetails.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
