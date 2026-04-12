import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import { useScyllaNavigate } from '@/modules/shared/presentation/hooks/useScyllaNavigate';

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
    },
  });
};
