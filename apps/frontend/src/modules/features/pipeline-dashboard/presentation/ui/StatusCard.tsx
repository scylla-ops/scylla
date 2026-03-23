import { Button } from '@/modules/shared/presentation/ui/shadcn';
import type { PipelineResponse } from '@/generated/pipeline.ts';
import { EditIcon, PlayIcon, MoreHorizontal, Clock } from 'lucide-react';
import StatusIndicator from '@/modules/shared/presentation/ui/status-indicator.tsx';
import { useNavigate } from 'react-router-dom';
import { PipelineChart } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineChart.tsx';
import { ListCard, type ListCardSection } from '@shared/presentation/ui/card-table/ListCard.tsx';
import type { SyntheticEvent } from 'react';

export type StatusCardProps = {
  pipeline: PipelineResponse;
  onClick?: () => void;
  selected?: boolean;
};

export const StatusCard = ({ pipeline, onClick, selected }: StatusCardProps) => {
  const navigate = useNavigate();

  const goToSettings = (e: SyntheticEvent) => {
    e.stopPropagation();
    navigate(`/pipeline-creation/${pipeline.pipelineId}`);
  };

  const handleRun = (e: SyntheticEvent) => {
    e.stopPropagation();
  };

  const creationDate = new Date(pipeline.createdAt);

  const sections: ListCardSection[] = [
    // STATUS
    {
      width: '20%',
      className: 'flex items-center gap-3 shrink-0',
      content: (
        <>
          <StatusIndicator state='running' />
          <div className='flex flex-col overflow-hidden'>
            <span className='font-semibold text-slate-900 truncate'>{pipeline.name}</span>
            <span className='text-xs font-mono text-slate-600 uppercase truncate'>
              main • a7f2e1
            </span>
            <span className='text-xs text-slate-500'>Creation: {creationDate.toDateString()}</span>
          </div>
        </>
      ),
    },
    // CHART
    {
      width: '35%',
      className: 'flex items-center justify-center shrink-0',
      content: (
        <div className='w-full'>
          <PipelineChart />
        </div>
      ),
    },

    // METADATA
    {
      width: '20%',
      className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
      content: (
        <>
          <div className='flex items-center gap-1.5'>
            <Clock className='w-3.5 h-3.5' />
            <span>1m 12s</span>
          </div>
          <span className='text-xs italic truncate'>2m ago</span>
        </>
      ),
    },

    // ACTIONS
    {
      className: 'flex  flex-1  justify-center items-center gap-1 shrink-0',
      content: (
        <>
          <Button
            size='icon'
            variant='ghost'
            className='h-8 w-8 text-slate-400 hover:text-primary hover:bg-indigo-50 rounded-full'
            onClick={handleRun}
          >
            <PlayIcon className='w-4 h-4 fill-current' />
          </Button>

          <Button
            size='icon'
            variant='ghost'
            className='h-8 w-8 text-slate-400 hover:text-slate-900 rounded-full'
            onClick={goToSettings}
          >
            <EditIcon className='w-4 h-4' />
          </Button>

          <Button size='icon' variant='ghost' className='h-8 w-8 text-slate-400 rounded-full'>
            <MoreHorizontal className='w-4 h-4' />
          </Button>
        </>
      ),
    },
  ];

  return <ListCard sections={sections} onClick={onClick} selected={selected} />;
};
