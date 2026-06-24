import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteTrigger } from '@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts';

interface TriggersHeaderProps {
  count: number;
  triggerIds: string[];
  pipelineId: string;
  onNew: () => void;
  isNew?: boolean;
}

export const TriggersHeader = ({ count, triggerIds, pipelineId, onNew, isNew }: TriggersHeaderProps) => {
  const deleteTrigger = useDeleteTrigger(pipelineId);
  const { headerProps } = useFeatureSelection('triggers', triggerIds, {
    deleteItem: id => deleteTrigger.mutateAsync(id),
  });

  return (
    <FeatureHeader
      count={count}
      label='Trigger'
      pluralLabel='Triggers'
      newLabel='New trigger'
      onNew={onNew}
      isNew={isNew}
      {...headerProps}
      underLabel={
        <span className='text-sm text-muted-foreground font-medium'>Pipeline ID: {pipelineId}</span>
      }
    />
  );
};
