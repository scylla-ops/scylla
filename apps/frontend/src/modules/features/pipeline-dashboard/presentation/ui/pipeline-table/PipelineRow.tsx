import type { PipelineResponse } from '@/generated/pipeline.ts';
import { useNavigate } from 'react-router-dom';
import { PipelineChart } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineChart.tsx';
import { ListCard, type ListCardSection } from '@shared/presentation/ui/ListCard.tsx';
import type { SyntheticEvent } from 'react';
import { getColumnConfig } from '@/modules/features/pipeline-dashboard/presentation/config/pipelineTableConfig.ts';
import { PipelineStatus, PipelineMetadata, PipelineActions } from './.';

export type StatusCardProps = {
  pipeline: PipelineResponse;
  onClick?: () => void;
  selected?: boolean;
};

/**
 * Component representing a single row in the pipeline dashboard, displaying the pipeline's status, history, metadata, and available actions.
 */
export const PipelineRow = ({ pipeline, onClick, selected }: StatusCardProps) => {
  const navigate = useNavigate();

  const handleEdit = (e: SyntheticEvent) => {
    e.stopPropagation();
    navigate(`/pipeline-creation/${pipeline.pipelineId}`);
  };

  const handleRun = (e: SyntheticEvent) => {
    e.stopPropagation();
    // TODO
  };

  const handleMore = (e: SyntheticEvent) => {
    e.stopPropagation();
    // TODO
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
          <PipelineChart />
        </div>
      ),
    },
    // METADATA
    {
      width: metadataConfig.width,
      className: metadataConfig.className,
      content: <PipelineMetadata duration='1m 12s' lastRun='2m ago' />,
    },
    // ACTIONS
    {
      width: actionsConfig.width,
      className: actionsConfig.className,
      noSeparator: actionsConfig.noSeparator,
      content: <PipelineActions onRun={handleRun} onEdit={handleEdit} onMore={handleMore} />,
    },
  ];

  return <ListCard sections={sections} onClick={onClick} selected={selected} />;
};
