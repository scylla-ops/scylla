import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/organization-remote.data-source.ts';
import type { OrganizationMember } from '@/modules/features/organization/domain/structs/organization-member.struct.ts';
import { GrpcOrganizationMemberMapper } from '@/modules/features/organization/infrastructure/repository/mappers/grpc-organization-member.mapper.ts';

export default class DefaultOrganizationRepository implements OrganizationRepository {
  constructor(private readonly remoteDataSource: OrganizationRemoteDataSource) {}

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getAll();
  }

  public getMine(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getMine();
  }

  public async listMembers(organizationId: string): Promise<ScyllaResult<OrganizationMember[]>> {
    return (await this.remoteDataSource.listMembers(organizationId)).map(members =>
      members.map(GrpcOrganizationMemberMapper.toDomain),
    );
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
}
