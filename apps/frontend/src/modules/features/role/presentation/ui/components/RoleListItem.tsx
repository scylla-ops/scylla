import { Badge } from '@shadcn';
import { Checkbox } from '@shadcn/checkbox.tsx';
import { ListCard } from '@shared/presentation/ui';
import { Trans } from '@lingui/react/macro';
import { ShieldCheck } from 'lucide-react';
import type { RoleEntity } from '@/modules/features/role/domain/entities/role.entity.ts';
import { scopeName } from '@/modules/features/role/presentation/utils/authz-labels.ts';

interface RoleListItemProps {
  role: RoleEntity;
  memberCount: number;
  active: boolean;
  selected: boolean;
  onOpen: () => void;
  onToggleSelect: () => void;
}

export const RoleListItem = ({
  role,
  memberCount,
  active,
  selected,
  onOpen,
  onToggleSelect,
}: RoleListItemProps) => {
  return (
    <ListCard
      onClick={onOpen}
      height='auto'
      className={
        active
          ? 'bg-primary/6 border-primary/40 hover:border-primary/40'
          : 'hover:border-slate-300 hover:shadow-sm'
      }
      sections={[
        {
          width: '20px',
          noSeparator: true,
          content: (
            <div className='flex items-center justify-center' onClick={e => e.stopPropagation()}>
              <Checkbox checked={selected} onCheckedChange={onToggleSelect} />
            </div>
          ),
        },
        {
          className: 'flex-1',
          noSeparator: true,
          content: (
            <div className='flex items-center gap-3 w-full min-w-0 overflow-hidden mr-4'>
              <div className='flex size-9 items-center justify-center rounded-lg bg-primary/10 shrink-0'>
                <ShieldCheck className='size-4 text-primary' />
              </div>
              <div className='flex flex-col items-start min-w-0 flex-1'>
                <p className='font-semibold text-foreground truncate w-full'>{role.name}</p>
                <p className='text-xs text-muted-foreground truncate w-full'>
                  {role.description || <Trans>No description</Trans>}
                </p>
              </div>
            </div>
          ),
        },
        {
          width: 'fit-content',
          noSeparator: true,
          content: (
            <div className='flex items-center  gap-1.5'>
              <Badge variant='secondary'>{scopeName(role.scope)}</Badge>
              <Badge variant='outline'>
                <Trans>{memberCount} members</Trans>
              </Badge>
            </div>
          ),
        },
      ]}
    />
  );
};
