import OrganizationRepositoryImpl from '@/modules/features/organization/infrastructure/repository/OrganizationRepositoryImpl.ts';
import GetOrganizations from '@/modules/features/organization/domain/usecases/GetOrganizations.ts';
import OrganizationRemoteDataSourceImpl from '@/modules/features/organization/infrastructure/data/OrganizationRemoteDataSourceImpl.ts';
import CreateOrganization from '@/modules/features/organization/domain/usecases/CreateOrganization.ts';
import { CoreModule } from '@core/di/CoreModule.ts';

const organizationRemoteDataSource = new OrganizationRemoteDataSourceImpl(
  CoreModule.data.grpcTransport,
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
