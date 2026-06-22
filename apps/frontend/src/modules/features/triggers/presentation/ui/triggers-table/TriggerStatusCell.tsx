import { Badge } from '@shadcn';
import { CheckCircle2, MinusCircle, XCircle } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { formatDate } from '@shared/utils/date-utils.ts';

/** Last-run badge (shared status language) plus a context line: next fire or disabled reason. */
export const TriggerStatusCell = ({ trigger }: { trigger: TriggerEntity }) => {
  const hasFired = trigger.lastStatus.length > 0 || !!trigger.lastFiredAt;
  const isError = trigger.lastStatus.startsWith('error');

  return (
    <div className='flex flex-col items-start gap-1'>
      {!hasFired ? (
        <Badge variant='outline' className='gap-1'>
          <MinusCircle className='size-3' /> <Trans>Never fired</Trans>
        </Badge>
      ) : isError ? (
        <Badge variant='destructive' className='gap-1'>
          <XCircle className='size-3' /> <Trans>Error</Trans>
        </Badge>
      ) : (
        <Badge variant='default' className='gap-1'>
          <CheckCircle2 className='size-3' /> <Trans>OK</Trans>
        </Badge>
      )}

      {!trigger.enabled ? (
        <span className='text-xs text-muted-foreground'>
          {trigger.source.kind === TriggerKind.Webhook ? (
            <Trans>disabled — URL returns 404</Trans>
          ) : (
            <Trans>disabled</Trans>
          )}
        </span>
      ) : (
        trigger.source.kind === TriggerKind.Cron &&
        trigger.nextFireAt && (
          <span className='text-xs text-muted-foreground'>
            <Trans>next</Trans> {formatDate(trigger.nextFireAt)}
          </span>
        )
      )}
    </div>
  );
};
