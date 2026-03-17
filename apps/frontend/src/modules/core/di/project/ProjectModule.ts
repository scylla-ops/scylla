import { ProjectRemoteDataSourceImpl } from '@/modules/features/project/infrastructure/data/ProjectRemoteDataSourceImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { ProjectRepositoryImpl } from '@/modules/features/project/infrastructure/repository/ProjectRepositoryImpl.ts';
import { GetProjects } from '@/modules/features/project/domain/use-cases/GetProjects.ts';
import { CreateProject } from '@/modules/features/project/domain/use-cases/CreateProject.ts';

const projectRemoteDataSource = new ProjectRemoteDataSourceImpl(CoreModule.data.coreGrpcTransport);
const projectRepository = new ProjectRepositoryImpl(projectRemoteDataSource);

const getProjectsUseCase = new GetProjects(projectRepository);
const createProjectUseCase = new CreateProject(projectRepository);

export const ProjectModule = {
  domain: { getProjects: getProjectsUseCase, createProject: createProjectUseCase },
};
