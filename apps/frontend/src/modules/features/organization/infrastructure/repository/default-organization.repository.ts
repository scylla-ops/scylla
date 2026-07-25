import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/organization-remote.data-source.ts';
import type { OrganizationMemberEntity } from '@/modules/features/organization/domain/entities/organization-member.entity.ts';

export default class DefaultOrganizationRepository implements OrganizationRepository {
  constructor(private readonly remoteDataSource: OrganizationRemoteDataSource) {}

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getAll();
  }

  public getMine(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getMine();
  }

  public create(name: string, description?: string): Promise<ScyllaResult<Organization>> {
    return this.remoteDataSource.create(name, description);
  }

  public update(
    organizationId: string,
    name?: string,
    description?: string,
  ): Promise<ScyllaResult<Organization>> {
    return this.remoteDataSource.update(organizationId, name, description);
  }

  public delete(organizationId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.delete(organizationId);
  }

  public listMembers(
    organizationId: string,
  ): Promise<ScyllaResult<OrganizationMemberEntity[]>> {
    return this.remoteDataSource.listMembers(organizationId);
  }

  public addMember(organizationId: string, userId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.addMember(organizationId, userId);
  }

  public removeMember(organizationId: string, userId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.removeMember(organizationId, userId);
  }
}
