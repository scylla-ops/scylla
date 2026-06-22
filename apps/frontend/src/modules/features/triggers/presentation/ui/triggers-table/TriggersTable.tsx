import { useState } from 'react';
import { DataTable } from '@shared/presentation/ui/data-display/DataTable.tsx';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { useDeleteTrigger } from '@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts';
import { useSetTriggerEnabled } from '@/modules/features/triggers/presentation/hooks/use-set-trigger-enabled.ts';
import { useFireTriggerNow } from '@/modules/features/triggers/presentation/hooks/use-fire-trigger-now.ts';
import { TriggerFormDialog } from '@/modules/features/triggers/presentation/ui/dialogs/TriggerFormDialog.tsx';
import { createTriggerColumns } from './columns.tsx';

interface TriggersTableProps {
  triggers: TriggerEntity[];
  pipelineId: string;
  projectId: string;
  pipelineName: string;
}

export const TriggersTable = ({
  triggers,
  pipelineId,
  projectId,
  pipelineName,
}: TriggersTableProps) => {
  const { selectedIds, select } = useSelection('triggers');
  const { goToJobs } = useScyllaNavigate();
  const deleteTrigger = useDeleteTrigger(pipelineId);
  const setEnabled = useSetTriggerEnabled(pipelineId);
  const fireNow = useFireTriggerNow(pipelineId);

  const [editTarget, setEditTarget] = useState<TriggerEntity | null>(null);
  const [deleteTargetId, setDeleteTargetId] = useState<string | null>(null);
  const [firingIds, setFiringIds] = useState<Set<string>>(new Set());

  const handleFire = (trigger: TriggerEntity) => {
    setFiringIds(prev => new Set(prev).add(trigger.id));
    fireNow
      .mutateAsync(trigger.id)
      // Close the loop: land on the run we just created.
      .then(() => goToJobs({ id: pipelineId, projectId, name: pipelineName }))
      .catch(() => {
        // Toast shown by the global MutationCache onError handler.
      })
      .finally(() =>
        setFiringIds(prev => {
          const next = new Set(prev);
          next.delete(trigger.id);
          return next;
        }),
      );
  };

  const columns = createTriggerColumns({
    onFire: handleFire,
    onEdit: trigger => setEditTarget(trigger),
    onDelete: triggerId => setDeleteTargetId(triggerId),
    onToggleEnabled: (trigger, enabled) => setEnabled.mutate({ triggerId: trigger.id, enabled }),
    firingIds,
  });

  return (
    <>
      <DataTable
        columns={columns}
        data={triggers}
        onRowClick={row => select(row.original.id)}
        getRowId={row => row.id}
        isRowSelected={row => selectedIds.includes(row.id)}
        alignColumnsCenter
      />

      {editTarget && (
        <TriggerFormDialog
          open={editTarget !== null}
          onOpenChange={open => {
            if (!open) setEditTarget(null);
          }}
          pipelineId={pipelineId}
          trigger={editTarget}
        />
      )}

      <ConfirmOperationAlertDialog
        open={deleteTargetId !== null}
        onOpenChange={open => {
          if (!open) setDeleteTargetId(null);
        }}
        onContinue={() => {
          if (!deleteTargetId) return;
          deleteTrigger.mutate(deleteTargetId);
          setDeleteTargetId(null);
        }}
      />
    </>
  );
};
