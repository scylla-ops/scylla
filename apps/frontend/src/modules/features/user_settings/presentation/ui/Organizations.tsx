import { Trans, useLingui } from '@lingui/react/macro';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@shadcn'; // Correction du path si tu as déplacé shadcn dans shared

import {
  useAddUserToOrganization,
  useOrganizationUsers,
  useRemoveUserFromOrganization,
  useUpdateUserRole,
} from '@/modules/features/user_settings/presentation/hooks/useUserSettings.ts';

import { AddUserDialog } from './AddUserDialog';
import { RemoveUserDialog } from './RemoveUserDialog';
import { EditRoleDialog } from './EditRoleDialog';
import { useContextStore } from '@shared/presentation/stores/useContext.ts';

export const Organizations = () => {
  const { i18n } = useLingui();

  const selectedOrgId = useContextStore(state => state.organization.id);

  const usersQuery = useOrganizationUsers(selectedOrgId || '');
  const addUserMutation = useAddUserToOrganization();
  const removeUserMutation = useRemoveUserFromOrganization();
  const updateRoleMutation = useUpdateUserRole();

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
        {/* Affichage des erreurs de mutations (Optionnel: tu peux utiliser un Toast global) */}
        {[addUserMutation, removeUserMutation, updateRoleMutation].map(
          (m, i) =>
            m.error && (
              <div key={i} className='bg-red-50 p-3 rounded text-red-800 text-sm'>
                {m.error.message}
              </div>
            ),
        )}

        {!selectedOrgId ? (
          <div className='text-center py-8 text-gray-500'>
            <Trans>Please select an organization first.</Trans>
          </div>
        ) : (
          <div className='space-y-4 border-t pt-6'>
            <div className='flex justify-between items-center'>
              <h3 className='text-lg font-semibold'>
                <Trans>Members</Trans>
              </h3>
              <AddUserDialog onAddUser={handleAddUser} isLoading={addUserMutation.isPending} />
            </div>

            {usersQuery.isLoading ? (
              <div className='flex justify-center py-8'>
                <Trans>Loading members...</Trans>
              </div>
            ) : usersQuery.isError ? (
              <div className='text-red-600'>
                <Trans>Error loading members</Trans>
              </div>
            ) : users.length === 0 ? (
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
                  {users.map(user => (
                    <TableRow key={user.user_id}>
                      <TableCell className='font-medium'>{user.username}</TableCell>
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
                          onUpdateRole={role => handleUpdateRole(user.user_id, role)}
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
