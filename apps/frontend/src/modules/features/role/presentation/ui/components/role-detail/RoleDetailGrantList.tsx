import { Trans } from '@lingui/react/macro';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { PrincipalKind } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { AppWindow, User, X } from 'lucide-react';
import { IconButton } from '@shared/presentation/ui';
import { useRoleAssignees } from '@/modules/features/role/presentation/hooks/use-role-assignees.ts';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import GrantCreator from '@/modules/features/role/presentation/ui/components/role-detail/GrantCreator.tsx';

interface RoleDetailGrantsProps {
  role: RoleEntity;
}
export const RoleDetailGrantList = ({ role }: RoleDetailGrantsProps) => {
  const { assignees, removeAssignee } = useRoleAssignees(role);

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
                    <p className='font-medium text-foreground truncate'>
                      {label} -{'>'} {grant.scopeId.length == 0 ? 'System' : grant.scopeId}
                    </p>
                    <p className='font-mono text-xs text-muted-foreground truncate'>
                      {grant.principal.id}
                    </p>
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
