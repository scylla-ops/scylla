import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/organization.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/organization-remote.data-source.ts';

export default class DefaultOrganizationRepository implements OrganizationRepository {
  constructor(private readonly remoteDataSource: OrganizationRemoteDataSource) {}

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getAll();
  }

  public create(name: string): Promise<ScyllaResult<OrganizationResponse>> {
    return this.remoteDataSource.create(name);
  }
}
