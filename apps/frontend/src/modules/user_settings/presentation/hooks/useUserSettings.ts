import { useMutation, useQuery } from '@tanstack/react-query';
import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';

export const useOrganizationUsers = (repo: UserSettingsRepository, organizationId: string) => {
  return useQuery({
    queryKey: ['organization-users', organizationId],
    queryFn: async () => {
      const result = await repo.getOrganizationUsers(organizationId);
      if (!result.ok) throw result.error;
      return result.value;
    },
    enabled: !!organizationId,
  });
};

export const useAddUserToOrganization = (repo: UserSettingsRepository) => {
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
      const result = await repo.addUserToOrganization(userId, organizationId, role);
      if (!result.ok) throw result.error;
      return result.value;
    },
  });
};

export const useRemoveUserFromOrganization = (repo: UserSettingsRepository) => {
  return useMutation({
    mutationFn: async ({ userId, organizationId }: { userId: string; organizationId: string }) => {
      const result = await repo.removeUserFromOrganization(userId, organizationId);
      if (!result.ok) throw result.error;
      return result.value;
    },
  });
};

export const useUpdateUserRole = (repo: UserSettingsRepository) => {
  return useMutation({
    mutationFn: async ({
      userId,
      organizationId,
      newRole,
    }: {
      userId: string;
      organizationId: string;
      newRole: string;
    }) => {
      const result = await repo.updateUserRole(userId, organizationId, newRole);
      if (!result.ok) throw result.error;
      return result.value;
    },
  });
};
