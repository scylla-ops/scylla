import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useDeletePipeline } from '@/modules/features/pipeline/presentation/hooks/use-delete-pipeline.ts';

interface PipelineDashboardHeaderProps {
  numberOfPipelines: number;
}

export const PipelineDashboardHeader = ({ numberOfPipelines }: PipelineDashboardHeaderProps) => {
  const { goToCreatePipeline } = useScyllaNavigate();
  const deletePipeline = useDeletePipeline();
  const { selectedIds, clearSelection } = useSelection('pipelines');

  const handleDelete = async () => {
    const promises = selectedIds.map(id => deletePipeline.mutateAsync(id));
    await Promise.all(promises);
    clearSelection();
  };

  return (
    <FeatureHeader
      count={numberOfPipelines}
      label='Pipeline'
      selectedCount={selectedIds.length}
      onClearSelection={clearSelection}
      onDeleteSelection={handleDelete}
      onNew={goToCreatePipeline}
      newLabel={<Trans>New pipeline</Trans>}
    />
  );
};
