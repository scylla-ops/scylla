import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/OrganizationRemoteDataSource.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse } from '@/generated/organization.ts';
import type { GrpcTransport } from '@core/infrastructure/grpc/GrpcTransport.ts';

export default class OrganizationRemoteDataSourceImpl implements OrganizationRemoteDataSource {
  private readonly _organizationClient: OrganizationServiceClient;

  constructor(transport: GrpcTransport) {
    this._organizationClient = new OrganizationServiceClient(transport.getTransport());
  }

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return ScyllaResult.tryAsync<ListOrganizationsResponse>(async () => {
      const { response } = await this._organizationClient.listOrganizations({});
      return response;
    }, 'Failed to fetch organizations.');
  }

  public create(name: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._organizationClient.createOrganization({ name });
    }, 'Failed to create organization.');
  }
}
