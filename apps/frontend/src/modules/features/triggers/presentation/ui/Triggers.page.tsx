import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { ErrorState } from '@/modules/shared/presentation/ui/feedback/ErrorState.tsx';
import { SecretRevealDialog } from '@shared/presentation/ui';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { usePipelineTriggers } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';
import {
  TriggersHeader,
  TriggersOverview,
} from '@/modules/features/triggers/presentation/ui/components/index.ts';
import { TriggersTable } from '@/modules/features/triggers/presentation/ui/triggers-table/index.ts';
import { TriggerFormDialog } from '@/modules/features/triggers/presentation/ui/dialogs/TriggerFormDialog.tsx';
import type { CreatedTrigger } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { useNewFeature } from '@shared/presentation/hooks/use-new-feature.ts';

interface RevealedSecret {
  id: string;
  name: string;
  secret: string;
}

export const TriggersPage = () => {
  const { pipelineId, projectId } = useParams<{ pipelineId: string; projectId: string }>();
  const pipelineName = useContextStore(state => state.pipeline?.name) ?? '';

  const { triggers, isLoading, isError, error } = usePipelineTriggers(pipelineId ?? '');
  const { isNew } = useNewFeature('triggers');

  const [isCreateOpen, setCreateOpen] = useState(false);
  const [revealed, setRevealed] = useState<RevealedSecret | null>(null);

  if (!pipelineId || !projectId) {
    return <ErrorState message='Pipeline ID is missing' />;
  }
  if (isLoading) {
    return <></>;
  }
  if (isError) {
    return <ErrorState message={error?.userMessage() ?? 'Unable to load triggers'} />;
  }

  const handleCreated = (created: CreatedTrigger) => {
    // Webhook secret is returned exactly once — reveal it now or it's lost.
    if (created.webhookSecret) {
      setRevealed({
        id: created.trigger.id,
        name: created.trigger.name,
        secret: created.webhookSecret,
      });
    }
  };

  return (
    <div className='flex flex-col gap-4 w-full min-h-full'>
      <TriggersHeader
        count={triggers.length}
        triggerIds={triggers.map(trigger => trigger.id)}
        pipelineId={pipelineId}
        onNew={() => setCreateOpen(true)}
        isNew={isNew}
      />

      {triggers.length > 0 && <TriggersOverview triggers={triggers} />}

      <div className='overflow-hidden'>
        {triggers.length > 0 ? (
          <TriggersTable
            triggers={triggers}
            pipelineId={pipelineId}
            projectId={projectId}
            pipelineName={pipelineName}
          />
        ) : (
          <div className='flex items-center justify-center min-h-[300px]'>
            <div className='text-center space-y-2'>
              <p className='text-muted-foreground'>
                <Trans>No triggers yet</Trans>
              </p>
              <p className='text-sm text-muted-foreground'>
                <Trans>Add a cron schedule or webhook to start runs automatically.</Trans>
              </p>
            </div>
          </div>
        )}
      </div>

      <TriggerFormDialog
        open={isCreateOpen}
        onOpenChange={setCreateOpen}
        pipelineId={pipelineId}
        onCreated={handleCreated}
      />

      {revealed && (
        <SecretRevealDialog
          open={true}
          entityKind='webhook'
          entity={{ id: revealed.id, name: revealed.name }}
          secret={revealed.secret}
          onClose={() => setRevealed(null)}
        />
      )}
    </div>
  );
};

export default TriggersPage;
