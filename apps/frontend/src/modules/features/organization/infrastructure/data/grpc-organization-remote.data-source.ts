import { OrganizationServiceClient } from '@/generated/scylla/organization/v1/organization.client.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListOrganizationsResponse,
  Organization,
  OrganizationMember,
} from '@/generated/scylla/organization/v1/organization.ts';
import type { ScyllaGrpcTransport } from '@platform/grpc';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import type { OrganizationRemoteDataSource } from '@/modules/features/organization/infrastructure/repository/data-sources/organization-remote.data-source.ts';

export default class GrpcOrganizationRemoteDataSource implements OrganizationRemoteDataSource {
  private readonly _organizationClient: OrganizationServiceClient;

  constructor(transport: ScyllaGrpcTransport) {
    this._organizationClient = new OrganizationServiceClient(transport.getTransport());
  }

  public getAll(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return ScyllaResult.tryAsync<ListOrganizationsResponse>(async () => {
      const { response } = await this._organizationClient.listOrganizations({});
      return response;
    }, 'Failed to fetch organizations.');
  }

  // The organizations of the current user: non-admins may not list them all.
  public getMine(): Promise<ScyllaResult<ListOrganizationsResponse>> {
    return ScyllaResult.tryAsync<ListOrganizationsResponse>(async () => {
      const userId = localStorage.getItem('userId') ?? '';
      const { response } = await this._organizationClient.listUserOrganizations({
        userId: wrapId(userId),
      });
      return { organizations: response.organizations, pagination: response.pagination };
    }, 'Failed to fetch organizations.');
  }

  /** Derived from grants: also lists people holding only a project grant in the organization. */
  public listMembers(organizationId: string): Promise<ScyllaResult<OrganizationMember[]>> {
    return ScyllaResult.tryAsync(async () => {
      const { response } = await this._organizationClient.listOrganizationMembers({
        organizationId: wrapId(organizationId),
      });
      return response.members;
    }, 'Failed to fetch organization members.');
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

/** The server always fills it on success: an absent one is a protocol error. */
function requireOrganization(organization?: Organization): Organization {
  if (!organization) throw new Error('Server returned no organization.');
  return organization;
}
