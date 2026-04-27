import { GrpcProjectRemoteDataSource } from '@/modules/features/project/infrastructure/data/grpc-project-remote.data-source.ts';
import { DefaultProjectRepository } from '@/modules/features/project/infrastructure/repository/default-project.repository.ts';
import { GetProjectsUseCase } from '@/modules/features/project/domain/use-cases/get-projects.use-case.ts';
import { CreateProjectUseCase } from '@/modules/features/project/domain/use-cases/create-project.use-case.ts';
import { CoreModule } from '@core/di/core.module.ts';

const projectRemoteDataSource = new GrpcProjectRemoteDataSource(CoreModule.data.grpcTransport);
const projectRepository = new DefaultProjectRepository(projectRemoteDataSource);

const getProjectsUseCase = new GetProjectsUseCase(projectRepository);
const createProjectUseCase = new CreateProjectUseCase(projectRepository);

export const ProjectModule = {
  domain: { getProjects: getProjectsUseCase, createProject: createProjectUseCase },
};
