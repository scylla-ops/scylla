import type { OrganizationRepository } from '@/modules/organization/domain/repository/OrganizationRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse } from '@/generated/organization.ts';
import type { OrganizationRemoteDataSource } from '@/modules/organization/repository/data-sources/OrganizationRemoteDataSource.ts';

export default class OrganizationRepositoryImpl implements OrganizationRepository {
  constructor(private readonly organizationRemoteDataSource: OrganizationRemoteDataSource) {}

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return this.organizationRemoteDataSource.getAll();
  }

  public create(name: string): Promise<ScyllaResult<void>> {
    return this.organizationRemoteDataSource.create(name);
  }
}
