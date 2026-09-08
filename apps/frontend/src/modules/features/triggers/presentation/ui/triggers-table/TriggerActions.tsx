import type { SyntheticEvent } from 'react';
import { EditIcon, Loader2, MoreHorizontal, PlayIcon, Trash2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { IconButton } from '@shared/presentation/ui';
import { useCompactContainer } from '@shared/presentation/hooks/use-compact-container.ts';
import { Permission } from '@platform/authz';
import { useCan } from '@platform/authz';

interface TriggerActionsProps {
  onFire: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onDelete: (e: SyntheticEvent) => void;
  isFiring?: boolean;
}

/**
 * Row actions: fire now (test), edit, delete. All three write to the trigger,
 * so they hide together behind `MANAGE_TRIGGERS`.
 *
 * Switches to dropdown mode when the column gets too narrow for three buttons.
 */
export const TriggerActions = ({ onFire, onEdit, onDelete, isFiring }: TriggerActionsProps) => {
  const canManage = useCan(Permission.MANAGE_TRIGGERS);
  const { containerRef, isCompact } = useCompactContainer();

  if (!canManage) return null;

  return (
    <div ref={containerRef} className='flex w-full shrink-0 items-center justify-center gap-1'>
      {isCompact ? (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type='button'
              size='icon'
              variant='ghost'
              className='h-8 w-8 shrink-0 rounded-full text-slate-400 hover:text-slate-900'
            >
              <MoreHorizontal className='h-4 w-4' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end' className='w-40'>
            <DropdownMenuItem onClick={onFire} disabled={isFiring}>
              <PlayIcon className='mr-2 h-4 w-4' />
              <Trans>Fire now</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onEdit}>
              <EditIcon className='mr-2 h-4 w-4' />
              <Trans>Edit</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onDelete} className='text-destructive'>
              <Trash2 className='mr-2 h-4 w-4' />
              <Trans>Delete</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      ) : (
        <>
          <IconButton
            icon={isFiring ? Loader2 : PlayIcon}
            tooltip={<Trans>Fire now</Trans>}
            onClick={onFire}
            disabled={isFiring}
            iconClassName={isFiring ? 'animate-spin' : 'fill-current'}
          />
          <IconButton icon={EditIcon} tooltip={<Trans>Edit</Trans>} onClick={onEdit} />
          <IconButton
            icon={Trash2}
            tooltip={<Trans>Delete</Trans>}
            onClick={onDelete}
            iconClassName='text-destructive'
          />
        </>
      )}
    </div>
  );
};
