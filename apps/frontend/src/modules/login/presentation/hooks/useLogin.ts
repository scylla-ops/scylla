import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies.ts';
import { useNavigate } from 'react-router-dom';
import type { ScyllaError } from '@core/utils/ScyllaResult.ts';

export const useLogin = () => {
  const deps = useDependencies();
  const navigate = useNavigate();

  return useMutation<void, ScyllaError, { login: string; password: string }>({
    mutationFn: async ({ login, password }) => {
      const result = await deps.login.loginUseCase.execute(login, password);
      return result.unwrap();
    },
    onSuccess: () => {
      navigate('/user-settings');
    },
    onError: err => {
      err.log();
    },
  });
};
