import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useDeleteTrigger } from '@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts';

interface TriggersHeaderProps {
  count: number;
  pipelineId: string;
  onNew: () => void;
}

export const TriggersHeader = ({ count, pipelineId, onNew }: TriggersHeaderProps) => {
  const { selectedIds, clearSelection } = useSelection('triggers');
  const deleteTrigger = useDeleteTrigger(pipelineId);

  const handleDelete = async () => {
    await Promise.allSettled(selectedIds.map(id => deleteTrigger.mutateAsync(id)));
    clearSelection();
  };

  return (
    <FeatureHeader
      count={count}
      label='Trigger'
      pluralLabel='Triggers'
      newLabel='New trigger'
      onNew={onNew}
      selectedCount={selectedIds.length}
      onClearSelection={clearSelection}
      onDeleteSelection={handleDelete}
      underLabel={
        <span className='text-sm text-muted-foreground font-medium'>Pipeline ID: {pipelineId}</span>
      }
    />
  );
};
