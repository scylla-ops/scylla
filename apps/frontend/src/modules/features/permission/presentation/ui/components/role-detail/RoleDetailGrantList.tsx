import { Trans } from '@lingui/react/macro';
import { Badge } from '@shadcn';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { AppWindow, Building2, FolderGit2, Globe, User, X } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { IconButton } from '@shared/presentation/ui';
import {
  PermissionScope,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { RoleEntity } from '@/modules/features/permission/domain/entities/role.entity.ts';
import { useRoleAssignees } from '@/modules/features/permission/presentation/hooks/use-role-assignees.ts';
import {
  useGrantTargetLabels,
  type GrantTargetLabel,
} from '@/modules/features/permission/presentation/hooks/use-grant-target-labels.ts';
import GrantCreator from '@/modules/features/permission/presentation/ui/components/role-detail/GrantCreator.tsx';

interface RoleDetailGrantsProps {
  role: RoleEntity;
}

const scopeIcon: Record<PermissionScope, LucideIcon> = {
  [PermissionScope.SYSTEM]: Globe,
  [PermissionScope.ORGANIZATION]: Building2,
  [PermissionScope.PROJECT]: FolderGit2,
  [PermissionScope.UNSPECIFIED]: Globe,
};

/** The scope target of a grant, resolved to a name, shown as an icon chip. */
const ScopeTargetBadge = ({
  scope,
  target,
}: {
  scope: PermissionScope;
  target: GrantTargetLabel;
}) => {
  const Icon = scopeIcon[scope] ?? Globe;
  return (
    <Badge variant='secondary' className='gap-1 font-normal max-w-full'>
      <Icon className='size-3 shrink-0' />
      <span className='truncate'>
        {target.organizationName ? `${target.organizationName} / ` : ''}
        {target.name}
      </span>
    </Badge>
  );
};

export const RoleDetailGrantList = ({ role }: RoleDetailGrantsProps) => {
  const { assignees, removeAssignee } = useRoleAssignees(role);
  const { labelFor } = useGrantTargetLabels(role.scope);

  return (
    <section className='flex flex-col gap-2 min-h-0'>
      <div className='flex items-center justify-between'>
        <h3 className='text-xs font-semibold uppercase tracking-wider text-slate-500'>
          <Trans>Grants</Trans> ({assignees.length})
        </h3>

        <GrantCreator role={role} />
      </div>
      {assignees.length === 0 ? (
        <p className='rounded-lg border border-dashed border-slate-200 py-6 text-center text-sm text-muted-foreground'>
          <Trans>No one holds this role yet.</Trans>
        </p>
      ) : (
        <ScrollArea className='max-h-72'>
          <ul className='flex flex-col gap-2 pr-2'>
            {assignees.map(({ grant, label }) => {
              const isUser = grant.principal.kind === PrincipalKind.USER;
              const target = labelFor(grant.scopeId);
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
                  <div className='flex flex-col items-start gap-1 min-w-0'>
                    <p className='font-medium text-foreground truncate max-w-full'>{label}</p>
                    <ScopeTargetBadge scope={role.scope} target={target} />
                  </div>
                  <IconButton
                    icon={X}
                    tooltip={<Trans>Remove</Trans>}
                    className='ml-auto hover:text-destructive'
                    onClick={() => removeAssignee(grant.id)}
                  />
                </li>
              );
            })}
          </ul>
        </ScrollArea>
      )}
    </section>
  );
};

export default RoleDetailGrantList;
