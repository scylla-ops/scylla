import { CoreModule } from '@core/di/core.module.ts';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import { GrpcJobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/data/remote/grpc-jobs-remote.data-source.ts';
import { DefaultJobsRepository } from '@/modules/features/jobs/infrastructure/repository/default-jobs.repository.ts';
import { GetPipelineJobsUseCase } from '@/modules/features/jobs/domain/usecases/get-pipeline-jobs.use-case.ts';
import { GetJobUseCase } from '@/modules/features/jobs/domain/usecases/get-job.use-case.ts';
import { DeleteJobsUseCase } from '@/modules/features/jobs/domain/usecases/delete-jobs.use-case.ts';
import { ListJobLogsUseCase } from '@/modules/features/jobs/domain/usecases/list-job-logs.use-case.ts';
import { TailJobLogsUseCase } from '@/modules/features/jobs/domain/usecases/tail-job-logs.use-case.ts';

const jobsRemoteDataSource: JobsRemoteDataSource = new GrpcJobsRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const jobsRepository = new DefaultJobsRepository(jobsRemoteDataSource);

const getPipelineJobs = new GetPipelineJobsUseCase(jobsRepository);
const getJobById = new GetJobUseCase(jobsRepository);
const deleteJob = new DeleteJobsUseCase(jobsRepository);
const listJobLogs = new ListJobLogsUseCase(jobsRepository);
const tailJobLogs = new TailJobLogsUseCase(jobsRepository);

export const JobsModule = {
  domain: { getPipelineJobs, getJobById, deleteJob, listJobLogs, tailJobLogs },
};
