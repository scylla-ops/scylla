import type { OrganizationRepository } from '@/modules/features/organization/domain/repository/OrganizationRepository.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/OrganizationRemoteDataSource.ts';

export default class OrganizationRepositoryImpl implements OrganizationRepository {
  constructor(private readonly remoteDataSource: OrganizationRemoteDataSource) {}

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.remoteDataSource.getAll();
  }

  public create(name: string): Promise<ScyllaResult<OrganizationResponse>> {
    return this.remoteDataSource.create(name);
  }
}
