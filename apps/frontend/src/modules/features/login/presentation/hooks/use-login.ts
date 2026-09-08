import { useMutation } from '@tanstack/react-query';
import { useLoginDomain } from '@/modules/features/login/presentation/hooks/use-login-domain.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import { useNavigate } from 'react-router-dom';

export const useLogin = () => {
  const { loginRepository } = useLoginDomain();
  const navigate = useNavigate();

  return useMutation<void, ScyllaError, { login: string; password: string }>({
    mutationFn: async ({ login, password }) => {
      const result = await loginRepository.login(login, password);
      return result.unwrap();
    },
    onSuccess: () => {
      void navigate('/', { replace: true });
    },
  });
};
