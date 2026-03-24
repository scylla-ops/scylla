import { Button } from '@/modules/shared/presentation/ui/shadcn';
import { EditIcon, PlayIcon, MoreHorizontal } from 'lucide-react';
import type { SyntheticEvent } from 'react';

type PipelineActionsProps = {
  onRun: (e: SyntheticEvent) => void;
  onEdit: (e: SyntheticEvent) => void;
  onMore?: (e: SyntheticEvent) => void;
};

/**
 * Display actions for a pipeline in the status card, such as run, edit, and more options.
 */
export const PipelineActions = ({ onRun, onEdit, onMore }: PipelineActionsProps) => {
  return (
    <>
      <Button
        size='icon'
        variant='ghost'
        className='h-8 w-8 text-slate-400 hover:text-primary hover:bg-primary-hover rounded-full'
        onClick={onRun}
      >
        <PlayIcon className='w-4 h-4 fill-current' />
      </Button>

      <Button
        size='icon'
        variant='ghost'
        className='h-8 w-8 text-slate-400 hover:text-slate-900 rounded-full'
        onClick={onEdit}
      >
        <EditIcon className='w-4 h-4' />
      </Button>

      <Button
        size='icon'
        variant='ghost'
        className='h-8 w-8 text-slate-400 rounded-full'
        onClick={onMore}
      >
        <MoreHorizontal className='w-4 h-4' />
      </Button>
    </>
  );
};
