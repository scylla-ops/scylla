import type { PipelineSummary } from '@/generated/pipeline.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { ListCard, type ListCardSection } from '@shared/presentation/ui';
import { PIPELINE_COLUMNS } from '@/modules/features/pipeline-dashboard/presentation/config/pipelineTableConfig.ts';
import { PipelineRow } from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineRow.tsx';
import { cn } from '@shared/presentation/utils';

type PipelineTableProps = {
  pipelines: PipelineSummary[];
};

const headerSections: ListCardSection[] = PIPELINE_COLUMNS.map(column => ({
  width: column.width,
  className: cn(
    'h-full flex justify-center items-center  gap-4 shrink-0 text-slate-500 text-sm',
    column.id === 'actions' && 'flex-1',
  ),
  noSeparator: column.noSeparator,
  content: (
    <span className='w-full h-full rounded-2xl hover:bg-primary-foreground hover:shadow flex items-center justify-center transition-transform hover:scale-110 text-xs font-semibold uppercase tracking-wider'>
      {column.label}
    </span>
  ),
}));

export const PipelineTable = ({ pipelines }: PipelineTableProps) => {
  const selectPipeline = usePipelineDashboardStore(state => state.selectPipeline);
  const selectedPipelineIds = usePipelineDashboardStore(state => state.selectedPipelineIds);

  return (
    <div className={'flex flex-col h-full gap-3'}>
      <ListCard sections={headerSections} className='hover:bg-transparent px-4 py-2 mb-4' />
      <div className='h-full flex flex-col gap-2'>
        {pipelines.map((pipeline, index) => (
          <PipelineRow
            key={index}
            selected={selectedPipelineIds.includes(pipeline.pipelineId)}
            onClick={() => {
              selectPipeline(pipeline.pipelineId);
            }}
            pipeline={pipeline}
          />
        ))}
      </div>
    </div>
  );
};
