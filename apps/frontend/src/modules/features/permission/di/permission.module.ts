import { CoreModule } from '@core/di/core.module.ts';
import { GrpcPermissionRemoteDataSource } from '@/modules/features/permission/infrastructure/data/grpc-permission-remote.data-source.ts';
import { DefaultPermissionRepository } from '@/modules/features/permission/infrastructure/repository/default-permission.repository.ts';
import { ListRolesUseCase } from '@/modules/features/permission/domain/usecases/list-roles.use-case.ts';
import { CreateRoleUseCase } from '@/modules/features/permission/domain/usecases/create-role.use-case.ts';
import { UpdateRoleUseCase } from '@/modules/features/permission/domain/usecases/update-role.use-case.ts';
import { DeleteRoleUseCase } from '@/modules/features/permission/domain/usecases/delete-role.use-case.ts';
import { GetEffectivePermissionsUseCase } from '@/modules/features/permission/domain/usecases/get-effective-permissions.use-case.ts';
import { GetMyPermissionsUseCase } from '@/modules/features/permission/domain/usecases/get-my-permissions.use-case.ts';
import { ListGrantsUseCase } from '@/modules/features/permission/domain/usecases/list-grants.use-case.ts';
import { CreateGrantUseCase } from '@/modules/features/permission/domain/usecases/create-grant.use-case.ts';
import { RevokeGrantUseCase } from '@/modules/features/permission/domain/usecases/revoke-grant.use-case.ts';
import { ListGrantableRolesUseCase } from '@/modules/features/permission/domain/usecases/list-grantable-roles.use-case.ts';
import { ListPermissionVocabularyUseCase } from '@/modules/features/permission/domain/usecases/list-permission-vocabulary.use-case.ts';

const dataSource = new GrpcPermissionRemoteDataSource(CoreModule.data.grpcTransport);
const repository = new DefaultPermissionRepository(dataSource);

export const PermissionModule = {
  domain: {
    listRoles: new ListRolesUseCase(repository),
    createRole: new CreateRoleUseCase(repository),
    updateRole: new UpdateRoleUseCase(repository),
    deleteRole: new DeleteRoleUseCase(repository),
    getEffectivePermissions: new GetEffectivePermissionsUseCase(repository),
    getMyPermissions: new GetMyPermissionsUseCase(repository),
    listGrants: new ListGrantsUseCase(repository),
    createGrant: new CreateGrantUseCase(repository),
    revokeGrant: new RevokeGrantUseCase(repository),
    listGrantableRoles: new ListGrantableRolesUseCase(repository),
    listPermissionVocabulary: new ListPermissionVocabularyUseCase(repository),
  },
};
