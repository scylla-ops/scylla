import { Button } from '@/modules/shared/presentation/ui/shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/modules/shared/presentation/ui/shadcn/dropdown-menu';
import { Eye, Trash, MoreHorizontal, TerminalSquare } from 'lucide-react';
import type { SyntheticEvent } from 'react';
import { useRef, useState, useEffect } from 'react';
import { Trans } from '@lingui/react/macro';
import { IconButton } from '@shared/presentation/ui';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

type JobActionsProps = {
  onView: (e: SyntheticEvent) => void;
  onDelete: (e: SyntheticEvent) => void;
  onOpenJobLog: (e: SyntheticEvent) => void;
};

/**
 * Display actions for a job: view details and delete
 * Automatically switches to dropdown mode when space is limited
 */
export const JobActions = ({ onView, onDelete, onOpenJobLog }: JobActionsProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isCompact, setIsCompact] = useState(false);

  const canViewLogs = useCan(Permission.READ_JOB_LOGS);
  const canDelete = useCan(Permission.DELETE_JOB);

  useEffect(() => {
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        setIsCompact(entry.contentRect.width < 70);
      }
    });

    if (containerRef.current) {
      observer.observe(containerRef.current);
    }

    return () => {
      observer.disconnect();
    };
  }, []);

  return (
    <div ref={containerRef} className='flex w-full items-center justify-center gap-2 shrink-0'>
      {isCompact ? (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type={'button'}
              size='icon'
              variant='ghost'
              className='h-8 w-8 shrink-0 text-slate-400 hover:text-slate-900 rounded-full'
            >
              <MoreHorizontal className='w-4 h-4' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end' className='w-40'>
            {onView && (
              <DropdownMenuItem onClick={onView}>
                <Eye className='w-4 h-4 mr-2' />
                <Trans>View</Trans>
              </DropdownMenuItem>
            )}
            {canDelete && (
              <DropdownMenuItem onClick={onDelete} className='text-destructive'>
                <Trash className='w-4 h-4 mr-2' />
                <Trans>Delete</Trans>
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      ) : (
        <>
          <IconButton icon={Eye} tooltip={<Trans>View</Trans>} onClick={onView} />

          {canViewLogs && (
            <IconButton
              icon={TerminalSquare}
              tooltip={<Trans>Open logs</Trans>}
              onClick={onOpenJobLog}
            />
          )}

          {canDelete && (
            <IconButton
              icon={Trash}
              tooltip={<Trans>Delete</Trans>}
              onClick={onDelete}
              className='hover:text-destructive hover:bg-destructive/10'
            />
          )}
        </>
      )}
    </div>
  );
};
