import { Button } from '@shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { EditIcon, PlayIcon, MoreHorizontal, ListChecks, Loader2, Copy, Zap } from 'lucide-react';
import type { SyntheticEvent } from 'react';
import { useRef, useState, useEffect } from 'react';
import { Trans } from '@lingui/react/macro';
import { IconButton } from '@shared/presentation/ui';

type PipelineActionsProps = {
  onRun: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onDuplicate: (e: SyntheticEvent) => void;
  onViewJobs?: (e: SyntheticEvent) => void;
  onViewTriggers?: (e: SyntheticEvent) => void;
  onMore?: (e: SyntheticEvent) => void;
  isRunning?: boolean;
};

/**
 * Display actions for a pipeline in the status card, such as run, edit, and more options.
 * Automatically switches to dropdown mode when space is limited.
 */
export const PipelineActions = ({
  onRun,
  onEdit,
  onDuplicate,
  onViewJobs,
  onViewTriggers,
  onMore,
  isRunning,
}: PipelineActionsProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isCompact, setIsCompact] = useState(false);

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
            <DropdownMenuItem onClick={onRun}>
              <PlayIcon className='w-4 h-4 mr-2' />
              <Trans>Run</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onEdit}>
              <EditIcon className='w-4 h-4 mr-2' />
              <Trans>Edit</Trans>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onDuplicate}>
              <Copy className='w-4 h-4 mr-2' />
              <Trans>Duplicate</Trans>
            </DropdownMenuItem>
            {onViewJobs && (
              <DropdownMenuItem onClick={onViewJobs}>
                <ListChecks className='w-4 h-4 mr-2' />
                View Jobs
              </DropdownMenuItem>
            )}
            {onViewTriggers && (
              <DropdownMenuItem onClick={onViewTriggers}>
                <Zap className='w-4 h-4 mr-2' />
                <Trans>Triggers</Trans>
              </DropdownMenuItem>
            )}
            {onMore && (
              <DropdownMenuItem onClick={onMore}>
                <MoreHorizontal className='w-4 h-4 mr-2' />
                <Trans>More options</Trans>
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      ) : (
        <>
          <IconButton
            icon={isRunning ? Loader2 : PlayIcon}
            tooltip={<Trans>Run</Trans>}
            onClick={onRun}
            iconClassName={isRunning ? 'animate-spin' : 'fill-current'}
          />

          <IconButton icon={EditIcon} tooltip={'Edit pipeline'} onClick={onEdit} />

          <IconButton icon={Copy} tooltip={<Trans>Duplicate</Trans>} onClick={onDuplicate} />

          {onViewJobs && (
            <IconButton icon={ListChecks} tooltip={<Trans>View Jobs</Trans>} onClick={onViewJobs} />
          )}

          {onViewTriggers && (
            <IconButton icon={Zap} tooltip={<Trans>Triggers</Trans>} onClick={onViewTriggers} />
          )}

          {onMore && (
            <IconButton
              icon={MoreHorizontal}
              tooltip={<Trans>More options</Trans>}
              onClick={onMore}
            />
          )}
        </>
      )}
    </div>
  );
};
