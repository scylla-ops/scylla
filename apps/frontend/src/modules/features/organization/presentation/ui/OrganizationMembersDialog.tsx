import { useEffect, useMemo, useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shadcn';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { Info, Trash, UserPlus } from 'lucide-react';
import { toast } from '@shared/presentation/utils/toast.ts';
import { IconButton } from '@shared/presentation/ui';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import {
  Permission,
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { PermissionButton } from '@/modules/features/permission/presentation/ui/authorization/PermissionButton.tsx';
import { useGrants } from '@/modules/features/permission/presentation/hooks/use-grants.ts';
import { useRoles } from '@/modules/features/permission/presentation/hooks/use-roles.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { useOrganizationMembers } from '@/modules/features/organization/presentation/hooks/use-organization-members.ts';
import type { OrganizationMember } from '@/modules/features/organization/domain/structs/organization-member.struct.ts';

/** The builtin whose whole content is "belongs here, sees it exists". */
const ORGANIZATION_MEMBER_ROLE_KEY = 'organization-member';

interface OrganizationMembersDialogProps {
  organization: { id: string; name: string } | null;
  onClose: () => void;
}

/**
 * Who belongs to an organization, and the two operations that change it.
 *
 * Membership has no storage of its own: the backend derives it from the grants
 * table, so this dialog does not call an "add member" or "remove member" RPC —
 * none exists, and none is missing. Admitting someone creates a grant at the
 * organization's scope, which is exactly what `AcceptInvitation` does when an
 * invitation is accepted. Removing them clears every grant they hold at that
 * scope *and beneath it*, in one `RevokeAllAccess`: revoking only the
 * organization-scoped grants would leave their project-scoped ones behind,
 * inert but still enough to keep them listed here.
 */
export const OrganizationMembersDialog = ({
  organization,
  onClose,
}: OrganizationMembersDialogProps) => {
  const { t } = useLingui();
  const organizationId = organization?.id ?? null;

  const { members, isLoading, refetchMembers } = useOrganizationMembers(organizationId);
  const { users } = useUsers();
  const { roles } = useRoles();
  const { createGrant, revokeAllAccess } = useGrants();

  const [userId, setUserId] = useState('');
  const [roleId, setRoleId] = useState('');
  const [pendingRemoval, setPendingRemoval] = useState<OrganizationMember | null>(null);

  /** Roles a grant at this organization may confer, member-role first. */
  const organizationRoles = useMemo(
    () => roles.filter(role => role.scope === PermissionScope.ORGANIZATION),
    [roles],
  );

  const defaultRoleId = useMemo(
    () =>
      organizationRoles.find(
        role => role.origin.kind === 'builtin' && role.origin.key === ORGANIZATION_MEMBER_ROLE_KEY,
      )?.id ?? '',
    [organizationRoles],
  );

  // Re-seed each time the dialog opens for an organization, and once the role
  // catalog has loaded enough to know what the default is.
  useEffect(() => {
    if (!organizationId) return;
    setUserId('');
    setPendingRemoval(null);
  }, [organizationId]);

  useEffect(() => {
    setRoleId(current => (current === '' ? defaultRoleId : current));
  }, [defaultRoleId]);

  const memberIds = useMemo(() => new Set(members.map(member => member.userId)), [members]);

  /** Only people not already in — re-admitting an existing member is a no-op. */
  const admittableUsers = useMemo(
    () => (users?.items ?? []).filter(user => !memberIds.has(user.userId)),
    [users, memberIds],
  );

  // Self-removal would revoke the caller's own access mid-session; the backend
  // may also refuse it as the organization's last owner. Excluded outright.
  const currentUserId = localStorage.getItem('userId') ?? '';

  const isPending = createGrant.isPending || revokeAllAccess.isPending;

  const handleAdd = async () => {
    if (!organizationId || userId === '' || roleId === '') return;
    try {
      await createGrant.mutateAsync({
        principal: { kind: PrincipalKind.USER, id: userId },
        roleId,
        scope: PermissionScope.ORGANIZATION,
        scopeId: organizationId,
      });
      refetchMembers();
      setUserId('');
      toast.success(t`Member added`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t`Failed to add the member`);
    }
  };

  const handleRemove = async () => {
    if (!organizationId || !pendingRemoval) return;
    const { username } = pendingRemoval;
    try {
      const revoked = await revokeAllAccess.mutateAsync({
        principal: { kind: PrincipalKind.USER, id: pendingRemoval.userId },
        scope: PermissionScope.ORGANIZATION,
        scopeId: organizationId,
      });
      refetchMembers();
      setPendingRemoval(null);
      toast.success(t`${username} removed — ${revoked} grant(s) revoked`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t`Failed to remove the member`);
    }
  };

  return (
    <>
      <Dialog open={!!organization} onOpenChange={value => !value && onClose()}>
        <DialogContent className='max-w-lg flex flex-col max-h-[85vh]'>
          <DialogHeader className='space-y-3'>
            <DialogTitle className='text-lg font-semibold'>
              <Trans>Members of “{organization?.name}”</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Belonging to an organization means holding a role in it. Adding someone grants them
                the role you pick; removing them revokes every role they hold here, including on its
                projects.
              </Trans>
            </DialogDescription>
          </DialogHeader>

          <div className='flex flex-col gap-4 overflow-y-auto pr-1'>
            <div className='flex flex-col gap-1.5'>
              <Label htmlFor='member-user'>
                <Trans>Add a member</Trans>
              </Label>
              <div className='flex gap-2'>
                <Select
                  value={userId}
                  disabled={isPending || admittableUsers.length === 0}
                  onValueChange={setUserId}
                >
                  <SelectTrigger id='member-user' className='flex-1'>
                    <SelectValue
                      placeholder={
                        admittableUsers.length === 0
                          ? t`Everyone is already a member`
                          : t`Select a user`
                      }
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {admittableUsers.map(user => (
                      <SelectItem key={user.userId} value={user.userId}>
                        {user.username}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Select value={roleId} disabled={isPending} onValueChange={setRoleId}>
                  <SelectTrigger id='member-role' className='flex-1'>
                    <SelectValue placeholder={t`Role`} />
                  </SelectTrigger>
                  <SelectContent>
                    {organizationRoles.map(role => (
                      <SelectItem key={role.id} value={role.id}>
                        {role.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Grants are administered system-wide in this build — the same
                  permission that gates the role-grant dialog gates this. */}
              <PermissionButton
                permission={Permission.MANAGE_SYSTEM_GRANTS}
                size='sm'
                className='self-start'
                disabled={userId === '' || roleId === '' || isPending}
                onClick={handleAdd}
                deniedReason={<Trans>You don't have permission to admit members.</Trans>}
              >
                <UserPlus className='size-4' />
                <Trans>Add</Trans>
              </PermissionButton>
            </div>

            <div className='flex flex-col gap-1.5'>
              <div className='flex items-center justify-between'>
                <Label>
                  <Trans>Members</Trans>
                </Label>
                <Badge variant='secondary'>{members.length}</Badge>
              </div>

              {isLoading ? (
                <p className='py-4 text-center text-sm text-muted-foreground'>
                  <Trans>Loading…</Trans>
                </p>
              ) : members.length === 0 ? (
                <p className='rounded-lg border border-dashed border-slate-200 py-4 text-center text-sm text-muted-foreground'>
                  <Trans>Nobody belongs to this organization yet.</Trans>
                </p>
              ) : (
                <ScrollArea className='max-h-64 rounded-lg border border-slate-200 p-2'>
                  <div className='flex flex-col gap-0.5'>
                    {members.map(member => (
                      <div
                        key={member.userId}
                        className='flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-slate-50'
                      >
                        <span className='flex-1 truncate text-sm'>{member.username}</span>
                        {member.userId === currentUserId ? (
                          <Badge variant='outline' className='text-[10px]'>
                            <Trans>You</Trans>
                          </Badge>
                        ) : (
                          <IconButton
                            icon={Trash}
                            tooltip={<Trans>Remove from the organization</Trans>}
                            disabled={isPending}
                            onClick={() => setPendingRemoval(member)}
                            className='h-7 w-7 hover:text-destructive hover:bg-destructive/10'
                            iconClassName='h-3.5 w-3.5'
                          />
                        )}
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              )}

              <p className='flex items-start gap-2 text-xs text-muted-foreground'>
                <Info className='size-3.5 shrink-0 mt-0.5' />
                <Trans>
                  Anyone holding a role on one of this organization's projects is listed here too.
                </Trans>
              </p>
            </div>
          </div>

          <DialogFooter>
            <Button type='button' variant='outline' disabled={isPending} onClick={onClose}>
              <Trans>Close</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmOperationAlertDialog
        open={!!pendingRemoval}
        onOpenChange={open => {
          if (!open) setPendingRemoval(null);
        }}
        isLoading={revokeAllAccess.isPending}
        title={
          <Trans>
            Remove {pendingRemoval?.username} from “{organization?.name}”?
          </Trans>
        }
        description={
          <Trans>
            Every role they hold on this organization and on its projects will be revoked. They will
            lose access immediately.
          </Trans>
        }
        onContinue={() => void handleRemove()}
      />
    </>
  );
};

export default OrganizationMembersDialog;
