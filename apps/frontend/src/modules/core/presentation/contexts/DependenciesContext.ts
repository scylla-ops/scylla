import { createContext } from 'react';

import type { LoginStore } from '@/modules/login/repository/store/LoginStore.ts';
import { LoginStoreImpl } from '@/modules/login/data/remote/LoginStoreImpl.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/login/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/login/domain/usecases/LoginUseCase.ts';

//todo: scind this by domain
class Dependencies {
  private loginStore: LoginStore = new LoginStoreImpl();
  private loginRepository: LoginRepository = new LoginRepositoryImpl(this.loginStore);

  public loginUseCase = new LoginUseCase(this.loginRepository);
}

export const DependenciesContext = createContext<Dependencies | null>(new Dependencies());
