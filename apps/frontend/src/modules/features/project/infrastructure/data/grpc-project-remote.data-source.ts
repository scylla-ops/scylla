import { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ProjectServiceClient } from '@/generated/project.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListProjectsResponse, ProjectResponse } from '@/generated/project.ts';
import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/project-remote.data-source.ts';
import {
  DEFAULT_PAGE_SIZE,
  type PaginationParams,
} from '@shared/domain/models/pagination.model.ts';

export class GrpcProjectRemoteDataSource implements GrpcProjectRemoteDataSource {
  private readonly _projectClient: ProjectServiceClient;

  constructor(_transport: CoreGrpcTransport) {
    this._projectClient = new ProjectServiceClient(_transport.getTransport());
  }

  public getByOrganizationId(
    organizationId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListProjectsResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.listOrganizationProjects({
        organizationId,
        pagination: pagination ?? { page: 1, pageSize: DEFAULT_PAGE_SIZE },
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

  public update(projectId: string, name?: string, description?: string): Promise<ScyllaResult<ProjectResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.updateProject({ projectId, name, description });
      return response;
    }, 'Failed to update project.');
  }

  public delete(projectId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._projectClient.deleteProject({ projectId });
    }, 'Failed to delete project.');
  }
}
