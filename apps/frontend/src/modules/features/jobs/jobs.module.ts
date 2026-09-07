import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { grpcTransport } from '@platform/grpc/index.ts';
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
} satisfies ScyllaModule;
