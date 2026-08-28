import { type CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ProjectServiceClient } from '@/generated/scylla/project/v1/project.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationProjectsResponse,
  Project,
  ProjectMember,
} from '@/generated/scylla/project/v1/project.ts';

import {
  DEFAULT_PAGE_SIZE,
  type PaginationParams,
} from '@shared/domain/structs/pagination.struct.ts';
import type { ProjectRemoteDataSource } from '@/modules/features/project/infrastructure/repository/data-sources/project-remote.data-source.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

/**
 * Every project RPC now answers with a `XxxResponse` wrapper holding the entity
 * in field 1. The unwrapping lives here so mappers and the domain keep seeing
 * plain `Project` entities.
 */
function requireProject(project: Project | undefined): Project {
  if (!project) throw new Error('Server returned no project.');
  return project;
}

export class GrpcProjectRemoteDataSource implements ProjectRemoteDataSource {
  private readonly _projectClient: ProjectServiceClient;

  constructor(_transport: CoreGrpcTransport) {
    this._projectClient = new ProjectServiceClient(_transport.getTransport());
  }

  public getByOrganizationId(
    organizationId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<ListOrganizationProjectsResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.listOrganizationProjects({
        organizationId: wrapId(organizationId),
        pagination: pagination ?? { page: 1, pageSize: DEFAULT_PAGE_SIZE },
      });
      return response;
    }, 'Failed to fetch projects.');
  }

  /**
   * The project's own members: the backend lists the holders of a grant scoped
   * to the project and nobody else, so someone reaching it through an
   * organization role is absent here by design.
   */
  public listMembers(projectId: string): Promise<ScyllaResult<ProjectMember[]>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.listProjectMembers({
        projectId: wrapId(projectId),
      });
      return response.members;
    }, 'Failed to fetch project members.');
  }

  public create(
    name: string,
    organizationId: string,
    description?: string,
  ): Promise<ScyllaResult<Project>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.createProject({
        name,
        organizationId: wrapId(organizationId),
        description,
      });
      return requireProject(response.project);
    }, 'Failed to create project.');
  }

  public update(
    projectId: string,
    name?: string,
    description?: string,
  ): Promise<ScyllaResult<Project>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._projectClient.updateProject({
        projectId: wrapId(projectId),
        name,
        description,
      });
      return requireProject(response.project);
    }, 'Failed to update project.');
  }

  public delete(projectId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._projectClient.deleteProject({ projectId: wrapId(projectId) });
    }, 'Failed to delete project.');
  }
}
