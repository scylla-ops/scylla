import { CoreModule } from '@core/di/CoreModule.ts';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/JobsRemoteDataSource.ts';
import { JobsRemoteDataSourceImpl } from '@/modules/features/jobs/infrastructure/data/remote/JobsRemoteDataSourceImpl.ts';
import { JobsRepositoryImpl } from '@/modules/features/jobs/infrastructure/repository/JobsRepositoryImpl.ts';
import { GetPipelineJobs } from '@/modules/features/jobs/domain/usecases/GetPipelineJobs.ts';
import { GetJobById } from '@/modules/features/jobs/domain/usecases/GetJobById.ts';
import { DeleteJob } from '@/modules/features/jobs/domain/usecases/DeleteJob.ts';

const jobsRemoteDataSource: JobsRemoteDataSource = new JobsRemoteDataSourceImpl(
  CoreModule.data.grpcTransport,
);
const jobsRepository = new JobsRepositoryImpl(jobsRemoteDataSource);

const getPipelineJobs = new GetPipelineJobs(jobsRepository);
const getJobById = new GetJobById(jobsRepository);
const deleteJob = new DeleteJob(jobsRepository);

export const JobsModule = {
  domain: { getPipelineJobs, getJobById, deleteJob },
};

