import { Button } from '@/modules/shared/presentation/ui/shadcn';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/modules/shared/presentation/ui/shadcn/dropdown-menu';
import { EditIcon, PlayIcon, MoreHorizontal } from 'lucide-react';
import type { SyntheticEvent } from 'react';
import { useRef, useState, useEffect } from 'react';

type PipelineActionsProps = {
  onRun: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onMore?: (e: SyntheticEvent) => void;
};

/**
 * Display actions for a pipeline in the status card, such as run, edit, and more options.
 * Automatically switches to dropdown mode when space is limited.
 */
export const PipelineActions = ({ onRun, onEdit, onMore }: PipelineActionsProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isCompact, setIsCompact] = useState(false);

  useEffect(() => {
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        // Si la largeur est inférieure à 140px, passer en mode compact
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
              Run
            </DropdownMenuItem>
            <DropdownMenuItem onClick={onEdit}>
              <EditIcon className='w-4 h-4 mr-2' />
              Edit
            </DropdownMenuItem>
            {onMore && (
              <DropdownMenuItem onClick={onMore}>
                <MoreHorizontal className='w-4 h-4 mr-2' />
                More options
              </DropdownMenuItem>
            )}
          </DropdownMenuContent>
        </DropdownMenu>
      ) : (
        <>
          <Button
            size='icon'
            variant='ghost'
            className='h-8 w-8 shrink-0 text-slate-400 hover:text-primary hover:bg-primary-hover rounded-full'
            onClick={onRun}
          >
            <PlayIcon className='w-4 h-4 fill-current' />
          </Button>

          <Button
            size='icon'
            variant='ghost'
            className='h-8 w-8 shrink-0 text-slate-400 hover:text-slate-900 rounded-full'
            onClick={onEdit}
          >
            <EditIcon className='w-4 h-4' />
          </Button>

          <Button
            size='icon'
            variant='ghost'
            className='h-8 w-8 shrink-0 text-slate-400 rounded-full'
            onClick={onMore}
          >
            <MoreHorizontal className='w-4 h-4' />
          </Button>
        </>
      )}
    </div>
  );
};
