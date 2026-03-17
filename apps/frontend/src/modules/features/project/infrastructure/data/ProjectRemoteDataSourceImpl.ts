import { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';
import { ProjectServiceClient } from '@/generated/project.client.ts';
import { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListProjectsResponse, ProjectResponse } from '@/generated/project.ts';
import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/ProjectRemoteDataSource.ts';

export class ProjectRemoteDataSourceImpl implements ProjectRemoteDataSource {
  private readonly _projectClient: ProjectServiceClient;

  constructor(_transport: CoreGrpcTransport) {
    this._projectClient = new ProjectServiceClient(_transport.getTransport());
  }

  public getAll(): Promise<ScyllaResult<ListProjectsResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.listProjects({
        pagination: { page: 1, pageSize: 10 },
      });
      return response;
    }, 'Failed to fetch projects.');
  }

  public create(name: string, organizationId: string): Promise<ScyllaResult<ProjectResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.createProject({ name, organizationId });
      return response;
    }, 'Failed to create project.');
  }
}
