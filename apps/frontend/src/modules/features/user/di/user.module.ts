import { UserRemoteDataSourceImpl } from '@/modules/features/user/infrastructure/data/remote/user-remote.data-source.impl.ts';
import { CoreModule } from '@core/di/CoreModule.ts';
import { UserRepositoryImpl } from '@/modules/features/user/infrastructure/repository/user-repository.ts';
import { GetUsersUseCase } from '@/modules/features/user/domain/use-cases/get-users.use-case.ts';
import { GetUserUseCase } from '@/modules/features/user/domain/use-cases/get-user.use-case.ts';
import { CreateUserUseCase } from '@/modules/features/user/domain/use-cases/create-user.use-case.ts';

const dataSource = new UserRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new UserRepositoryImpl(dataSource);

const getUsersUseCase = new GetUsersUseCase(repository);
const getUserUseCase = new GetUserUseCase(repository);
const createUserUseCase = new CreateUserUseCase(repository);

export const UserModule = {
  domain: { getUsers: getUsersUseCase, getUser: getUserUseCase, createUser: createUserUseCase },
};
