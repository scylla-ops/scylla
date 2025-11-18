import { useMutation } from '@tanstack/react-query';
import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies.ts';
import { useNavigate } from 'react-router-dom';

export const useLogin = () => {
  const deps = useDependencies();
  const navigate = useNavigate();

  return useMutation({
    mutationFn: async ({ login, password }: { login: string; password: string }) => {
      const result = await deps.login.loginUseCase.execute(login, password);
      if (!result.ok) throw result.error;
      return result.value;
    },
    onSuccess: token => {
      deps.core.setTokenUseCase.execute(token);
      navigate('/user_settings');
    },
    onError: err => {
      console.error('Erreur login:', err);
    },
  });
};
