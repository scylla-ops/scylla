import React, { useState, useEffect } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/modules/core/presentation/ui/shadcn';
import { UserSettingsModule } from '@/modules/user_settings/di/UserSettingsModule.ts';
import {
  useAddUserToOrganization,
  useOrganizationUsers,
  useRemoveUserFromOrganization,
  useUpdateUserRole,
} from '@/modules/user_settings/presentation/hooks/useUserSettings.ts';
import { AddUserDialog } from '@/modules/user_settings/presentation/ui/AddUserDialog.tsx';
import { RemoveUserDialog } from '@/modules/user_settings/presentation/ui/RemoveUserDialog.tsx';
import { EditRoleDialog } from '@/modules/user_settings/presentation/ui/EditRoleDialog.tsx';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';

// TODO: Get real organizations list from API
const SAMPLE_ORGANIZATIONS = [
  { id: 'org-1', name: 'Alpha organization' },
  { id: 'org-2', name: 'Beta organization' },
  { id: 'org-3', name: 'Zeta organization' },
];

export const Organizations = () => {
  const { i18n } = useLingui();
  const [selectedOrgId, setSelectedOrgId] = useState<string>(SAMPLE_ORGANIZATIONS[0]?.id || '');

  // Queries and mutations
  const usersQuery = useOrganizationUsers(UserSettingsModule.repository, selectedOrgId);
  const addUserMutation = useAddUserToOrganization(UserSettingsModule.repository);
  const removeUserMutation = useRemoveUserFromOrganization(UserSettingsModule.repository);
  const updateRoleMutation = useUpdateUserRole(UserSettingsModule.repository);

  // Refetch users when mutations succeed
  useEffect(() => {
    if (
      addUserMutation.isSuccess ||
      removeUserMutation.isSuccess ||
      updateRoleMutation.isSuccess
    ) {
      usersQuery.refetch();
    }
  }, [
    addUserMutation.isSuccess,
    removeUserMutation.isSuccess,
    updateRoleMutation.isSuccess,
    usersQuery,
  ]);

  const handleAddUser = (userId: string, role: string) => {
    if (selectedOrgId) {
      addUserMutation.mutate({ userId, organizationId: selectedOrgId, role });
    }
  };

  const handleRemoveUser = (userId: string) => {
    if (selectedOrgId) {
      removeUserMutation.mutate({ userId, organizationId: selectedOrgId });
    }
  };

  const handleUpdateRole = (userId: string, newRole: string) => {
    if (selectedOrgId) {
      updateRoleMutation.mutate({ userId, organizationId: selectedOrgId, newRole });
    }
  };

  const users = usersQuery.data?.users ?? [];

  return (
    <Card className='w-full bg-white'>
      <CardHeader>
        <CardTitle>
          <Trans>Organizations</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>Manage your organizations and their members.</Trans>
        </CardDescription>
      </CardHeader>

      <CardContent className='space-y-4'>
        <div className='grid gap-2'>
          <label htmlFor='org-select'>
            <Trans>Select Organization</Trans>
          </label>
          <Select value={selectedOrgId} onValueChange={setSelectedOrgId}>
            <SelectTrigger id='org-select'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {SAMPLE_ORGANIZATIONS.map((org) => (
                <SelectItem key={org.id} value={org.id}>
                  {org.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {selectedOrgId && (
          <div className='space-y-4 border-t pt-6'>
            <div className='flex justify-between items-center'>
              <h3 className='text-lg font-semibold'>
                <Trans>Members</Trans>
              </h3>
              <AddUserDialog
                onAddUser={handleAddUser}
                isLoading={addUserMutation.isPending}
              />
            </div>

            {addUserMutation.error && (
              <div className='bg-red-50 p-3 rounded text-red-800'>
                {(addUserMutation.error as any).message || 'Error adding user'}
              </div>
            )}

            {removeUserMutation.error && (
              <div className='bg-red-50 p-3 rounded text-red-800'>
                {(removeUserMutation.error as any).message || 'Error removing user'}
              </div>
            )}

            {updateRoleMutation.error && (
              <div className='bg-red-50 p-3 rounded text-red-800'>
                {(updateRoleMutation.error as any).message || 'Error updating role'}
              </div>
            )}

            {usersQuery.isLoading && <div><Trans>Loading members...</Trans></div>}

            {usersQuery.isError && <div><Trans>Error loading members</Trans></div>}

            {users.length === 0 && !usersQuery.isLoading ? (
              <div className='text-center text-gray-500 py-8'>
                <Trans>No members in this organization yet</Trans>
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>
                      <Trans>Username</Trans>
                    </TableHead>
                    <TableHead>
                      <Trans>Role</Trans>
                    </TableHead>
                    <TableHead>
                      <Trans>Joined</Trans>
                    </TableHead>
                    <TableHead className='text-right'>
                      <Trans>Actions</Trans>
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.map((user: OrganizationUser) => (
                    <TableRow key={user.user_id}>
                      <TableCell>{user.username}</TableCell>
                      <TableCell>
                        <span className='inline-flex items-center rounded-full bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700'>
                          {user.role}
                        </span>
                      </TableCell>
                      <TableCell>
                        {new Date(user.joined_at).toLocaleDateString(i18n.locale)}
                      </TableCell>
                      <TableCell className='text-right space-x-2'>
                        <EditRoleDialog
                          user={user}
                          onUpdateRole={(newRole) => handleUpdateRole(user.user_id, newRole)}
                          isLoading={updateRoleMutation.isPending}
                        />
                        <RemoveUserDialog
                          username={user.username}
                          onConfirm={() => handleRemoveUser(user.user_id)}
                          isLoading={removeUserMutation.isPending}
                        />
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};
