import { CoreModule } from '@core/di/core.module.ts';
import { GrpcAuthzRemoteDataSource } from '@/modules/features/authz/infrastructure/data/grpc-authz-remote.data-source.ts';
import { ListRolesUseCase } from '@/modules/features/authz/domain/usecases/list-roles.use-case.ts';
import { CreateRoleUseCase } from '@/modules/features/authz/domain/usecases/create-role.use-case.ts';
import { UpdateRoleUseCase } from '@/modules/features/authz/domain/usecases/update-role.use-case.ts';
import { DeleteRoleUseCase } from '@/modules/features/authz/domain/usecases/delete-role.use-case.ts';
import { GetEffectivePermissionsUseCase } from '@/modules/features/authz/domain/usecases/get-effective-permissions.use-case.ts';
import { ListGrantsUseCase } from '@/modules/features/authz/domain/usecases/list-grants.use-case.ts';
import { CreateGrantUseCase } from '@/modules/features/authz/domain/usecases/create-grant.use-case.ts';
import { RevokeGrantUseCase } from '@/modules/features/authz/domain/usecases/revoke-grant.use-case.ts';
import { ListGrantableRolesUseCase } from '@/modules/features/authz/domain/usecases/list-grantable-roles.use-case.ts';

// The gRPC data source implements AuthzRepository directly, so it is the
// repository the use cases depend on.
const repository = new GrpcAuthzRemoteDataSource(CoreModule.data.grpcTransport);

export const AuthzModule = {
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
  },
};
