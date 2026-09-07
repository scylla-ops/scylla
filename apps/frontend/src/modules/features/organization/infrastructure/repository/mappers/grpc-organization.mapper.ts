import type { Organization as GrpcOrganization } from '@/generated/scylla/organization/v1/organization.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import type { OrganizationEntity } from '@/modules/features/organization/domain/entities/organization.entity.ts';

export class GrpcOrganizationMapper {
  public static toDomain(organization: GrpcOrganization): OrganizationEntity {
    return {
      id: idValue(organization.organizationId),
      name: organization.name,
      description: organization.description,
    };
  }
}
