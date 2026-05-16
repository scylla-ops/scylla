import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import { useNavigate } from 'react-router-dom';

export const useLogin = () => {
  const deps = useDependencies();
  const navigate = useNavigate();

  return useMutation<void, ScyllaError, { login: string; password: string }>({
    mutationFn: async ({ login, password }) => {
      const result = await deps.login.loginUseCase.execute(login, password);
      return result.unwrap();
    },
    onSuccess: () => {
      navigate('/', { replace: true });
    },
  });
};
