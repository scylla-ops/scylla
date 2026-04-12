import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/OrganizationRemoteDataSource.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

export default class OrganizationRemoteDataSourceImpl implements OrganizationRemoteDataSource {
  private readonly _organizationClient: OrganizationServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._organizationClient = new OrganizationServiceClient(transport.getTransport());
  }

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return ScyllaResult.tryAsync<ListOrganizationsResponse>(async () => {
      const { response } = await this._organizationClient.listOrganizations({});
      return response;
    }, 'Failed to fetch organizations.');
  }

  public create(name: string): Promise<ScyllaResult<OrganizationResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.createOrganization({ name });
      return response;
    }, 'Failed to create organization.');
  }
}
