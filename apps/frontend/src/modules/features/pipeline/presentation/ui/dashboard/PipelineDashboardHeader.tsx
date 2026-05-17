import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader, BackButton } from '@shared/presentation/ui';
import { useDeletePipeline } from '@/modules/features/pipeline/presentation/hooks/use-delete-pipeline.ts';

interface PipelineDashboardHeaderProps {
  numberOfPipelines: number;
  onBack: () => void;
}

export const PipelineDashboardHeader = ({
  numberOfPipelines,
  onBack,
}: PipelineDashboardHeaderProps) => {
  const { goToCreatePipeline } = useScyllaNavigate();
  const deletePipeline = useDeletePipeline();
  const { selectedIds, clearSelection } = useSelection('pipelines');

  const handleDelete = async () => {
    const promises = selectedIds.map(id => deletePipeline.mutateAsync(id));
    await Promise.allSettled(promises);
    clearSelection();
  };

  return (
    <div className='flex items-center gap-4 w-full'>
      <BackButton iconOnly onClick={onBack} />
      <FeatureHeader
        count={numberOfPipelines}
        label='Pipeline'
        selectedCount={selectedIds.length}
        onClearSelection={clearSelection}
        onDeleteSelection={handleDelete}
        onNew={goToCreatePipeline}
        newLabel={<Trans>New pipeline</Trans>}
      />
    </div>
  );
};
