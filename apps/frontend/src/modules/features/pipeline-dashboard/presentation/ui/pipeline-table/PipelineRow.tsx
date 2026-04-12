import type { PipelineSummary } from '@/generated/pipeline.ts';
import { PipelineChart } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineChart.tsx';
import { ListCard, type ListCardSection } from '@shared/presentation/ui/ListCard.tsx';
import type { SyntheticEvent } from 'react';
import { getColumnConfig } from '@/modules/features/pipeline-dashboard/presentation/config/pipelineTableConfig.ts';
import { PipelineStatus, PipelineMetadata, PipelineActions } from './.';
import { useScyllaNavigate } from '@shared/presentation/hooks/useScyllaNavigate.ts';
import { useRunPipeline } from '../../hooks/useRunPipeline';

export type StatusCardProps = {
  pipeline: PipelineSummary;
  onClick?: () => void;
  selected?: boolean;
};

/**
 * Component representing a single row in the pipeline dashboard, displaying the pipeline's status, history, metadata, and available actions.
 */
export const PipelineRow = ({ pipeline, onClick, selected }: StatusCardProps) => {
  const { goToEditPipeline, goToJobs } = useScyllaNavigate();
  const runPipeline = useRunPipeline();

  const handleEdit = (e: SyntheticEvent) => {
    e.stopPropagation();
    goToEditPipeline(pipeline);
  };

  const handleRun = (e: SyntheticEvent) => {
    e.stopPropagation();
    runPipeline.mutateAsync(pipeline.pipelineId);
  };

  const handleViewJobs = (e: SyntheticEvent) => {
    e.stopPropagation();
    goToJobs(pipeline.pipelineId);
  };

  const statusConfig = getColumnConfig('status');
  const historyConfig = getColumnConfig('history');
  const metadataConfig = getColumnConfig('metadata');
  const actionsConfig = getColumnConfig('actions');

  const sections: ListCardSection[] = [
    // STATUS
    {
      width: statusConfig.width,
      className: statusConfig.className,
      content: <PipelineStatus pipeline={pipeline} />,
    },
    // HISTORY
    {
      width: historyConfig.width,
      className: historyConfig.className,
      content: (
        <div className='w-full'>
          <PipelineChart pipelineId={pipeline.pipelineId} />
        </div>
      ),
    },
    // METADATA
    {
      width: metadataConfig.width,
      className: metadataConfig.className,
      content: <PipelineMetadata pipelineId={pipeline.pipelineId} />,
    },
    // ACTIONS
    {
      width: actionsConfig.width,
      className: actionsConfig.className,
      noSeparator: actionsConfig.noSeparator,
      content: (
        <PipelineActions onRun={handleRun} onEdit={handleEdit} onViewJobs={handleViewJobs} />
      ),
    },
  ];

  return <ListCard sections={sections} onClick={onClick} selected={selected} />;
};
