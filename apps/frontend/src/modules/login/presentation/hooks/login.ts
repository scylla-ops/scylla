import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies.ts';

export const useLogin = () => {
  const deps = useDependencies();

  return useMutation({
    mutationFn: ({ login, password }: { login: string; password: string }) =>
      deps.loginUseCase.execute(login, password),
  });
};
