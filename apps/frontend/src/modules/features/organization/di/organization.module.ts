import DefaultOrganizationRepository from '@/modules/features/organization/infrastructure/repository/default-organization.repository.ts';
import GetOrganizationsUseCase from '@/modules/features/organization/domain/usecases/get-organizations.use-case.ts';
import GetUserOrganizationsUseCase from '@/modules/features/organization/domain/usecases/get-user-organizations.use-case.ts';
import GrpcOrganizationRemoteDataSource from '@/modules/features/organization/infrastructure/data/grpc-organization-remote.data-source.ts';
import CreateOrganizationUseCase from '@/modules/features/organization/domain/usecases/create-organization.use-case.ts';
import UpdateOrganizationUseCase from '@/modules/features/organization/domain/usecases/update-organization.use-case.ts';
import DeleteOrganizationUseCase from '@/modules/features/organization/domain/usecases/delete-organization.use-case.ts';
import ListOrganizationMembersUseCase from '@/modules/features/organization/domain/usecases/list-organization-members.use-case.ts';
import { CoreModule } from '@core/di/core.module.ts';

const organizationRemoteDataSource = new GrpcOrganizationRemoteDataSource(
  CoreModule.data.grpcTransport,
);

const organizationRepository = new DefaultOrganizationRepository(organizationRemoteDataSource);
const getOrganizationsUseCase = new GetOrganizationsUseCase(organizationRepository);
const getUserOrganizationsUseCase = new GetUserOrganizationsUseCase(organizationRepository);
const createOrganizationUseCase = new CreateOrganizationUseCase(organizationRepository);
const updateOrganizationUseCase = new UpdateOrganizationUseCase(organizationRepository);
const deleteOrganizationUseCase = new DeleteOrganizationUseCase(organizationRepository);
const listOrganizationMembersUseCase = new ListOrganizationMembersUseCase(organizationRepository);

export const OrganizationModule = {
  domain: {
    getOrganizations: getOrganizationsUseCase,
    getUserOrganizations: getUserOrganizationsUseCase,
    createOrganization: createOrganizationUseCase,
    updateOrganization: updateOrganizationUseCase,
    deleteOrganization: deleteOrganizationUseCase,
    listOrganizationMembers: listOrganizationMembersUseCase,
  },
};
