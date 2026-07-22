import { useMemo } from 'react';
import {
  Badge,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { AppWindow, User, Users } from 'lucide-react';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { PrincipalKind } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useGrants } from '@/modules/features/role/presentation/hooks/use-grants.ts';
import { principalKindName } from '@/modules/features/role/presentation/utils/authz-labels.ts';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';

interface RoleAssigneesDialogProps {
  role: RoleEntity | null;
  onClose: () => void;
}

export const RoleAssigneesDialog = ({ role, onClose }: RoleAssigneesDialogProps) => {
  const { grants } = useGrants();
  const { users } = useUsers();

  const usernameById = useMemo(
    () => new Map((users?.items ?? []).map(user => [user.userId, user.username])),
    [users],
  );

  const assignees = useMemo(
    () =>
      grants.filter(
        grant => grant.target.kind === 'role' && grant.target.roleId === role?.id,
      ),
    [grants, role?.id],
  );

  return (
    <Dialog open={role !== null} onOpenChange={open => !open && onClose()}>
      <DialogContent className='max-w-lg flex flex-col'>
        <DialogHeader className='space-y-3'>
          <DialogTitle className='flex items-center gap-2.5 text-lg font-semibold'>
            <div className='flex items-center justify-center size-8 rounded-lg bg-primary/10'>
              <Users className='size-4 text-primary' />
            </div>
            <span>
              <Trans>Assignees of {role?.name ?? ''}</Trans>
            </span>
          </DialogTitle>
          <DialogDescription>
            <Trans>Users and apps that hold this role.</Trans>
          </DialogDescription>
        </DialogHeader>

        {assignees.length === 0 ? (
          <p className='py-6 text-center text-sm text-muted-foreground'>
            <Trans>No one is assigned to this role.</Trans>
          </p>
        ) : (
          <ul className='flex flex-col gap-2 max-h-96 overflow-auto'>
            {assignees.map(grant => {
              const isUser = grant.principal.kind === PrincipalKind.USER;
              const label = isUser
                ? usernameById.get(grant.principal.id) ?? grant.principal.id
                : grant.principal.id;
              return (
                <li
                  key={grant.id}
                  className='flex items-center gap-3 rounded-lg border border-slate-200 px-3 py-2'
                >
                  <div className='flex size-9 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
                    {isUser ? (
                      <User className='size-4 text-primary' />
                    ) : (
                      <AppWindow className='size-4 text-primary' />
                    )}
                  </div>
                  <div className='flex flex-col items-start min-w-0'>
                    <p className='font-medium text-foreground truncate'>{label}</p>
                    <p className='font-mono text-xs text-muted-foreground truncate'>
                      ID: {grant.principal.id}
                    </p>
                  </div>
                  <Badge variant='secondary' className='ml-auto'>
                    {principalKindName(grant.principal.kind)}
                  </Badge>
                </li>
              );
            })}
          </ul>
        )}
      </DialogContent>
    </Dialog>
  );
};
