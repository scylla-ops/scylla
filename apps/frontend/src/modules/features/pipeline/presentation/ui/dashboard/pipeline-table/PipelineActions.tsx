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
import { useNewFeature } from '@shared/presentation/hooks/use-new-feature.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

type PipelineActionsProps = {
  onRun: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onDuplicate: (e: SyntheticEvent) => void;
  onViewJobs?: (e: SyntheticEvent) => void;
  onViewTriggers?: (e: SyntheticEvent) => void;
  onMore?: (e: SyntheticEvent) => void;
  isRunning?: boolean;
  isDuplicating?: boolean;
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
  isDuplicating,
}: PipelineActionsProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isCompact, setIsCompact] = useState(false);
  const { isNew: isTriggersNew } = useNewFeature('triggers');

  // Gate each action by the permission it needs, in the current project context.
  const canRun = useCan(Permission.RUN_PIPELINE);
  const canEdit = useCan(Permission.UPDATE_PIPELINE);
  const canDuplicate = useCan(Permission.CREATE_PIPELINE);
  const canManageTriggers = useCan(Permission.MANAGE_TRIGGERS);
  const showTriggers = !!onViewTriggers && canManageTriggers;

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
            {canRun && (
              <DropdownMenuItem onClick={onRun}>
                <PlayIcon className='w-4 h-4 mr-2' />
                <Trans>Run</Trans>
              </DropdownMenuItem>
            )}
            {canEdit && (
              <DropdownMenuItem onClick={onEdit}>
                <EditIcon className='w-4 h-4 mr-2' />
                <Trans>Edit</Trans>
              </DropdownMenuItem>
            )}
            {canDuplicate && (
              <DropdownMenuItem onClick={onDuplicate}>
                <Copy className='w-4 h-4 mr-2' />
                <Trans>Duplicate</Trans>
              </DropdownMenuItem>
            )}
            {onViewJobs && (
              <DropdownMenuItem onClick={onViewJobs}>
                <ListChecks className='w-4 h-4 mr-2' />
                View Jobs
              </DropdownMenuItem>
            )}
            {showTriggers && (
              <DropdownMenuItem onClick={onViewTriggers}>
                <Zap className='w-4 h-4 mr-2' />
                <Trans>Triggers</Trans>
                {isTriggersNew && (
                  <span className='relative inline-flex ml-2'>
                    <span className='absolute inset-0 animate-ping rounded-full bg-primary opacity-40' />
                    <span className='relative bg-primary text-primary-foreground text-[10px] font-bold px-1.5 py-0.5 rounded-full leading-none'>
                      New
                    </span>
                  </span>
                )}
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
          {canRun && (
            <IconButton
              icon={isRunning ? Loader2 : PlayIcon}
              tooltip={<Trans>Run</Trans>}
              onClick={onRun}
              disabled={isRunning}
              iconClassName={isRunning ? 'animate-spin' : 'fill-current'}
            />
          )}

          {canEdit && <IconButton icon={EditIcon} tooltip={'Edit pipeline'} onClick={onEdit} />}

          {canDuplicate && (
            <IconButton
              icon={isDuplicating ? Loader2 : Copy}
              tooltip={<Trans>Duplicate</Trans>}
              onClick={onDuplicate}
              disabled={isDuplicating}
              iconClassName={isDuplicating ? 'animate-spin' : undefined}
            />
          )}

          {onViewJobs && (
            <IconButton icon={ListChecks} tooltip={<Trans>View Jobs</Trans>} onClick={onViewJobs} />
          )}

          {showTriggers && (
            <div className='relative'>
              <IconButton icon={Zap} tooltip={<Trans>Triggers</Trans>} onClick={onViewTriggers} />
              {isTriggersNew && (
                <span className='absolute -top-1.5 -right-1.5 flex pointer-events-none'>
                  <span className='absolute inset-0 animate-ping rounded-full bg-primary opacity-40' />
                  <span className='relative bg-primary text-primary-foreground text-[10px] font-bold px-1 py-0.5 rounded-full leading-none'>
                    New
                  </span>
                </span>
              )}
            </div>
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
