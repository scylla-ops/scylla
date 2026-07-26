import { useMemo, useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { Plus, User, UserPlus, X } from 'lucide-react';
import { IconButton } from '@shared/presentation/ui';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { useOrganizationMembers } from '@/modules/features/organization/presentation/hooks/use-organization-members.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { PermissionButton } from '@/modules/features/permission/presentation/ui/authorization/PermissionButton.tsx';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

interface OrganizationMembersDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  organizationId: string;
  organizationName: string;
}

export const OrganizationMembersDialog = ({
  open,
  onOpenChange,
  organizationId,
  organizationName,
}: OrganizationMembersDialogProps) => {
  const { t } = useLingui();
  const { members, isLoading, addMember, removeMember } = useOrganizationMembers(
    open ? organizationId : null,
  );
  const { users } = useUsers();

  const [selectedUserId, setSelectedUserId] = useState('');

  const memberIds = useMemo(() => new Set(members.map(m => m.userId)), [members]);

  // Users not yet in the organization — candidates for the add picker.
  const addableUsers = useMemo(
    () => (users?.items ?? []).filter(user => !memberIds.has(user.userId)),
    [users, memberIds],
  );

  const handleAdd = () => {
    if (!selectedUserId) return;
    addMember.mutate(selectedUserId, { onSuccess: () => setSelectedUserId('') });
  };

  const isBusy = addMember.isPending || removeMember.isPending;
  const canRemove = useCan(Permission.REMOVE_ORGANIZATION_MEMBER, { organizationId });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='max-w-lg flex flex-col max-h-[85vh]'>
        <DialogHeader className='space-y-3'>
          <DialogTitle className='flex items-center gap-2.5 text-lg font-semibold'>
            <div className='flex items-center justify-center size-8 rounded-lg bg-primary/10'>
              <UserPlus className='size-4 text-primary' />
            </div>
            <span>
              <Trans>Members of {organizationName}</Trans>
            </span>
          </DialogTitle>
          <DialogDescription>
            <Trans>Add or remove the users that belong to this organization.</Trans>
          </DialogDescription>
        </DialogHeader>

        {/* Add a member */}
        <div className='flex items-end gap-2'>
          <div className='flex-1 min-w-0'>
            <Select
              value={selectedUserId}
              disabled={isBusy || addableUsers.length === 0}
              onValueChange={setSelectedUserId}
            >
              <SelectTrigger className='w-full'>
                <SelectValue
                  placeholder={
                    addableUsers.length === 0 ? t`Everyone is a member` : t`Select a user to add`
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {addableUsers.map(user => (
                  <SelectItem key={user.userId} value={user.userId}>
                    {user.username}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <PermissionButton
            type='button'
            permission={Permission.ADD_ORGANIZATION_MEMBER}
            target={{ organizationId }}
            deniedReason={<Trans>You don't have permission to add members.</Trans>}
            disabled={!selectedUserId || isBusy}
            onClick={handleAdd}
          >
            <Plus className='size-4' />
            <Trans>Add</Trans>
          </PermissionButton>
        </div>

        {/* Member list */}
        {isLoading ? (
          <p className='py-6 text-center text-sm text-muted-foreground'>
            <Trans>Loading members…</Trans>
          </p>
        ) : members.length === 0 ? (
          <p className='rounded-lg border border-dashed border-slate-200 py-6 text-center text-sm text-muted-foreground'>
            <Trans>No members yet.</Trans>
          </p>
        ) : (
          <ScrollArea className='max-h-72'>
            <ul className='flex flex-col gap-2 pr-2'>
              {members.map(member => (
                <li
                  key={member.userId}
                  className='flex items-center gap-3 rounded-lg border border-slate-200 px-3 py-2'
                >
                  <div className='flex size-9 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
                    <User className='size-4 text-primary' />
                  </div>
                  <div className='flex flex-col items-start min-w-0'>
                    <p className='font-medium text-foreground truncate'>{member.username}</p>
                    <p className='font-mono text-xs text-muted-foreground truncate'>
                      {member.userId}
                    </p>
                  </div>
                  <IconButton
                    icon={X}
                    tooltip={
                      canRemove ? (
                        <Trans>Remove</Trans>
                      ) : (
                        <Trans>You don't have permission to remove members.</Trans>
                      )
                    }
                    disabled={isBusy || !canRemove}
                    className='ml-auto hover:text-destructive'
                    onClick={() => removeMember.mutate(member.userId)}
                  />
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
      </DialogContent>
    </Dialog>
  );
};

export default OrganizationMembersDialog;
