import DefaultOrganizationRepository from '@/modules/features/organization/infrastructure/repository/default-organization.repository.ts';
import GetOrganizationsUseCase from '@/modules/features/organization/domain/usecases/get-organizations.use-case.ts';
import GrpcOrganizationRemoteDataSource from '@/modules/features/organization/infrastructure/data/grpc-organization-remote.data-source.ts';
import CreateOrganizationUseCase from '@/modules/features/organization/domain/usecases/create-organization.use-case.ts';
import { CoreModule } from '@core/di/core.module.ts';

const organizationRemoteDataSource = new GrpcOrganizationRemoteDataSource(
  CoreModule.data.grpcTransport,
);

const organizationRepository = new DefaultOrganizationRepository(organizationRemoteDataSource);
const getOrganizationsUseCase = new GetOrganizationsUseCase(organizationRepository);
const createOrganizationUseCase = new CreateOrganizationUseCase(organizationRepository);

export const OrganizationModule = {
  domain: {
    getOrganizations: getOrganizationsUseCase,
    createOrganization: createOrganizationUseCase,
  },
};
