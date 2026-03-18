import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useOrganizationUsers = (organizationId: string) => {
  const { getOrganizationUsers } = useDependencies().userSettings;
  return useQuery({
    queryKey: ['organization-users', organizationId],
    queryFn: async () => (await getOrganizationUsers.execute(organizationId)).unwrap(),
    enabled: !!organizationId,
  });
};

export const useAddUserToOrganization = () => {
  const queryClient = useQueryClient();
  const { addUserToOrganization } = useDependencies().userSettings;

  return useMutation({
    mutationFn: async ({
      userId,
      organizationId,
      role,
    }: {
      userId: string;
      organizationId: string;
      role: string;
    }) => {
      const result = await addUserToOrganization.execute(userId, organizationId, role);
      return result.unwrap();
    },

    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['organization-users', variables.organizationId],
      });
    },

    onError: error => {
      console.error("Erreur lors de l'ajout de l'utilisateur:", error.message);
    },
  });
};

export const useRemoveUserFromOrganization = () => {
  const { removeUserFromOrganization } = useDependencies().userSettings;

  return useMutation({
    mutationFn: async ({ userId, organizationId }: { userId: string; organizationId: string }) =>
      (await removeUserFromOrganization.execute(userId, organizationId)).unwrap(),
  });
};

export const useUpdateUserRole = () => {
  const { updateUserRole } = useDependencies().userSettings;
  return useMutation({
    mutationFn: async ({
      userId,
      organizationId,
      newRole,
    }: {
      userId: string;
      organizationId: string;
      newRole: string;
    }) => (await updateUserRole.execute(userId, organizationId, newRole)).unwrap(),
  });
};
