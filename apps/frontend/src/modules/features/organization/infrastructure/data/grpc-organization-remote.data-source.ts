import { OrganizationServiceClient } from '@/generated/scylla/organization/v1/organization.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

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

  // Member-scoped list: orgs the current user belongs to. Non-admins are
  // denied listOrganizations (global), so this is the default for the switcher.
  public getMine(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return ScyllaResult.tryAsync<ListOrganizationsResponse>(async () => {
      const userId = localStorage.getItem('userId') ?? '';
      const { response } = await this._organizationClient.listUserOrganizations({
        userId: wrapId(userId),
      });
      return { organizations: response.organizations, pagination: response.pagination };
    }, 'Failed to fetch organizations.');
  }

  public create(name: string, description?: string): Promise<ScyllaResult<Organization>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.createOrganization({ name, description });
      return requireOrganization(response.organization);
    }, 'Failed to create organization.');
  }

  public update(
    organizationId: string,
    name?: string,
    description?: string,
  ): Promise<ScyllaResult<Organization>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.updateOrganization({
        organizationId: wrapId(organizationId),
        name,
        description,
      });
      return requireOrganization(response.organization);
    }, 'Failed to update organization.');
  }

  public delete(organizationId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._organizationClient.deleteOrganization({ organizationId: wrapId(organizationId) });
    }, 'Failed to delete organization.');
  }
}

/**
 * Every organization RPC now answers with a wrapper message holding an optional
 * `organization`. The server always fills it on success, so an absent entity is
 * a protocol violation, not a state the rest of the app should model.
 */
function requireOrganization(organization?: Organization): Organization {
  if (!organization) throw new Error('Server returned no organization.');
  return organization;
}
