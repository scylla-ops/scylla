import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListOrganizationsResponse, OrganizationResponse } from '@/generated/organization.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';

export default class GrpcOrganizationRemoteDataSource implements GrpcOrganizationRemoteDataSource {
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

  public create(name: string, description?: string): Promise<ScyllaResult<OrganizationResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.createOrganization({ name, description });
      return response;
    }, 'Failed to create organization.');
  }

  public update(
    organizationId: string,
    name?: string,
    description?: string,
  ): Promise<ScyllaResult<OrganizationResponse>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.updateOrganization({
        organizationId,
        name,
        description,
      });
      return response;
    }, 'Failed to update organization.');
  }

  public delete(organizationId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._organizationClient.deleteOrganization({ organizationId });
    }, 'Failed to delete organization.');
  }
}
