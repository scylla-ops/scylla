import { CoreModule } from '@core/di/core/CoreModule.ts';
import OrganizationRepositoryImpl from '@/modules/organization/repository/OrganizationRepositoryImpl.ts';
import GetOrganizations from '@/modules/organization/domain/usecases/GetOrganizations.ts';
import OrganizationRemoteDataSourceImpl from '@/modules/organization/data/OrganizationRemoteDataSourceImpl.ts';
import CreateOrganization from '@/modules/organization/domain/usecases/CreateOrganization.ts';

const organizationRemoteDataSource = new OrganizationRemoteDataSourceImpl(
  CoreModule.data.coreGrpcTransport,
);

const organizationRepository = new OrganizationRepositoryImpl(organizationRemoteDataSource);
const getOrganizationsUseCase = new GetOrganizations(organizationRepository);
const createOrganizationUseCase = new CreateOrganization(organizationRepository);

export const OrganizationModule = {
  domain: {
    getOrganizations: getOrganizationsUseCase,
    createOrganization: createOrganizationUseCase,
  },
};
