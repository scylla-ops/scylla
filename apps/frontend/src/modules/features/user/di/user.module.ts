import { UserRemoteDataSourceImpl } from '@/modules/features/user/infrastructure/data/remote/user-remote.data-source.impl.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultUserRepository } from '@/modules/features/user/infrastructure/repository/default-user.repository.ts';
import { GetUsersUseCase } from '@/modules/features/user/domain/use-cases/get-users.use-case.ts';
import { GetUserUseCase } from '@/modules/features/user/domain/use-cases/get-user.use-case.ts';
import { CreateUserUseCase } from '@/modules/features/user/domain/use-cases/create-user.use-case.ts';
import { DeleteUserUseCase } from '@/modules/features/user/domain/use-cases/delete-user.use-case.ts';
import { UpdateUserUseCase } from '@/modules/features/user/domain/use-cases/update-user.use-case.ts';

const dataSource = new UserRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultUserRepository(dataSource);

const getUsersUseCase = new GetUsersUseCase(repository);
const getUserUseCase = new GetUserUseCase(repository);
const createUserUseCase = new CreateUserUseCase(repository);
const updateUserUseCase = new UpdateUserUseCase(repository);
const deleteUserUseCase = new DeleteUserUseCase(repository);

export const UserModule = {
  domain: {
    getUsers: getUsersUseCase,
    getUser: getUserUseCase,
    createUser: createUserUseCase,
    updateUser: updateUserUseCase,
    deleteUser: deleteUserUseCase,
  },
};
