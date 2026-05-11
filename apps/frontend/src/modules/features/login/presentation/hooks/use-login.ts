import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

export const useLogin = () => {
  const deps = useDependencies();
  const goToUserSettings = useScyllaNavigate().goToUserSettings;

  return useMutation<void, ScyllaError, { login: string; password: string }>({
    mutationFn: async ({ login, password }) => {
      const result = await deps.login.loginUseCase.execute(login, password);
      return result.unwrap();
    },
    onSuccess: () => {
      goToUserSettings();
      //reset navigation history
    },
  });
};
