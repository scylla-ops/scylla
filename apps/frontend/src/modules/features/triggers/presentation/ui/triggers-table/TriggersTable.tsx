import { useState } from 'react';
import { DataTable } from '@shared/presentation/ui/data-display/DataTable.tsx';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Permission, useCan } from '@platform/authz';
import { Trans, useLingui } from '@lingui/react/macro';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import { useScyllaNavigate } from '@platform/context';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { useDeleteTrigger } from '@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts';
import { useSetTriggerEnabled } from '@/modules/features/triggers/presentation/hooks/use-set-trigger-enabled.ts';
import { useFireTriggerNow } from '@/modules/features/triggers/presentation/hooks/use-fire-trigger-now.ts';
import { TriggerFormDialog } from '@/modules/features/triggers/presentation/ui/dialogs/TriggerFormDialog.tsx';
import { createTriggerColumns } from './columns.tsx';

interface TriggersTableProps {
  triggers: TriggerEntity[];
  pipelineId: string;
  pipelineName: string;
}

export const TriggersTable = ({ triggers, pipelineId, pipelineName }: TriggersTableProps) => {
  const deleteTrigger = useDeleteTrigger(pipelineId);
  const { selectedIds, select, headerProps } = useFeatureSelection(
    'triggers',
    triggers.map(trigger => trigger.id),
    { deleteItem: id => deleteTrigger.mutateAsync(id) },
  );
  // Triggers are all-or-nothing in V1: one permission covers create and delete.
  const canManage = useCan(Permission.MANAGE_TRIGGERS);
  const { t } = useLingui();
  const { goToJobs } = useScyllaNavigate();
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
      .then(() => goToJobs(pipelineId, pipelineName))
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
        searchable
        searchPlaceholder={t`Search a trigger…`}
        searchValues={trigger => [trigger.name, trigger.id]}
        facets={[
          {
            id: 'source',
            label: t`Source`,
            value: trigger => trigger.source.kind,
            options: [
              { value: TriggerKind.Cron, label: t`Cron` },
              { value: TriggerKind.Webhook, label: t`Webhook` },
            ],
          },
          {
            id: 'enabled',
            label: t`State`,
            value: trigger => (trigger.enabled ? 'enabled' : 'disabled'),
            options: [
              { value: 'enabled', label: t`Enabled` },
              { value: 'disabled', label: t`Disabled` },
            ],
          },
        ]}
        selection={{
          ...headerProps,
          canDelete: canManage,
          deleteDeniedReason: <Trans>You don't have permission to manage triggers.</Trans>,
        }}
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
