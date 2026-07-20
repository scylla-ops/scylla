import { Badge } from '@shadcn';
import { CheckCircle2, HelpCircle, MinusCircle, XCircle } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { formatDate } from '@shared/utils/date-utils.ts';

/** Last-run badge (shared status language) plus a context line: next fire or disabled reason. */
export const TriggerStatusCell = ({ trigger }: { trigger: TriggerEntity }) => {
  const result = trigger.lastResult;
  const hasFired = !!result || !!trigger.lastFiredAt;

  return (
    <div className='flex flex-col items-center justify-center gap-1'>
      {!hasFired ? (
        <Badge variant='outline' className='gap-1'>
          <MinusCircle className='size-3' /> <Trans>Never fired</Trans>
        </Badge>
      ) : result?.kind === 'failed' ? (
        <Badge variant='destructive' className='gap-1' title={result.error}>
          <XCircle className='size-3' /> <Trans>Error</Trans>
        </Badge>
      ) : result?.kind === 'succeeded' ? (
        <Badge variant='default' className='gap-1'>
          <CheckCircle2 className='size-3' /> <Trans>OK</Trans>
        </Badge>
      ) : (
        // Fired, but the outcome arm is newer than this build — don't claim success.
        <Badge variant='outline' className='gap-1'>
          <HelpCircle className='size-3' /> <Trans>Unknown</Trans>
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
      ) : trigger.source.kind === TriggerKind.Cron ? (
        trigger.nextFireAt && (
          <span className='text-xs text-muted-foreground'>
            <Trans>next</Trans> {formatDate(trigger.nextFireAt)}
          </span>
        )
      ) : (
        trigger.lastFiredAt && (
          <span className='text-xs text-muted-foreground'>
            <Trans>last</Trans> {formatDate(trigger.lastFiredAt)}
          </span>
        )
      )}
    </div>
  );
};
