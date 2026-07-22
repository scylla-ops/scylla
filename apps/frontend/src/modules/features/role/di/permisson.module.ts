import { CoreModule } from '@core/di/core.module.ts';
import { GrpcPermissionRemoteDataSource } from '@/modules/features/role/infrastructure/data/grpc-permission-remote.data-source.ts';
import { DefaultPermissionRepository } from '@/modules/features/role/infrastructure/repository/default-permission.repository.ts';
import { ListRolesUseCase } from '@/modules/features/role/domain/usecases/list-roles.use-case.ts';
import { CreateRoleUseCase } from '@/modules/features/role/domain/usecases/create-role.use-case.ts';
import { UpdateRoleUseCase } from '@/modules/features/role/domain/usecases/update-role.use-case.ts';
import { DeleteRoleUseCase } from '@/modules/features/role/domain/usecases/delete-role.use-case.ts';
import { GetEffectivePermissionsUseCase } from '@/modules/features/role/domain/usecases/get-effective-permissions.use-case.ts';
import { ListGrantsUseCase } from '@/modules/features/role/domain/usecases/list-grants.use-case.ts';
import { CreateGrantUseCase } from '@/modules/features/role/domain/usecases/create-grant.use-case.ts';
import { RevokeGrantUseCase } from '@/modules/features/role/domain/usecases/revoke-grant.use-case.ts';
import { ListGrantableRolesUseCase } from '@/modules/features/role/domain/usecases/list-grantable-roles.use-case.ts';
import { ListAuthzVocabularyUseCase } from '@/modules/features/role/domain/usecases/list-authz-vocabulary.use-case.ts';

const dataSource = new GrpcPermissionRemoteDataSource(CoreModule.data.grpcTransport);
const repository = new DefaultPermissionRepository(dataSource);

export const PermissonModule = {
  domain: {
    listRoles: new ListRolesUseCase(repository),
    createRole: new CreateRoleUseCase(repository),
    updateRole: new UpdateRoleUseCase(repository),
    deleteRole: new DeleteRoleUseCase(repository),
    getEffectivePermissions: new GetEffectivePermissionsUseCase(repository),
    listGrants: new ListGrantsUseCase(repository),
    createGrant: new CreateGrantUseCase(repository),
    revokeGrant: new RevokeGrantUseCase(repository),
    listGrantableRoles: new ListGrantableRolesUseCase(repository),
    listAuthzVocabulary: new ListAuthzVocabularyUseCase(repository),
  },
};
