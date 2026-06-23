import { Card, CardContent } from '@shadcn';
import { Clock, Power, Webhook } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { formatDate } from '@shared/utils/date-utils.ts';

/** A calm at-a-glance summary row, modeled on the secrets health overview. */
export const TriggersOverview = ({ triggers }: { triggers: TriggerEntity[] }) => {
  const enabledCount = triggers.filter(trigger => trigger.enabled).length;
  const webhookCount = triggers.filter(
    trigger => trigger.source.kind === TriggerKind.Webhook,
  ).length;
  const nextFire = triggers
    .filter(
      trigger =>
        trigger.enabled && trigger.source.kind === TriggerKind.Cron && !!trigger.nextFireAt,
    )
    .map(trigger => trigger.nextFireAt as string)
    .sort((a, b) => a.localeCompare(b))[0];

  return (
    <div className='grid gap-4 md:grid-cols-3'>
      <Card className='rounded-2xl border-primary/20 bg-primary/5 py-3'>
        <CardContent className='flex items-center gap-3'>
          <Power className='size-4 text-primary' />
          <div>
            <p className='text-2xl font-semibold tracking-tight'>
              {enabledCount}/{triggers.length}
            </p>
            <p className='text-xs text-muted-foreground'>
              <Trans>enabled</Trans>
            </p>
          </div>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-blue-200/40 bg-blue-500/5 py-3'>
        <CardContent className='flex items-center gap-3'>
          <Clock className='size-4 text-blue-500' />
          <div>
            <p className='text-sm font-semibold tracking-tight'>
              {nextFire ? formatDate(nextFire) : '—'}
            </p>
            <p className='text-xs text-muted-foreground'>
              <Trans>next scheduled run (Local time)</Trans>
            </p>
          </div>
        </CardContent>
      </Card>

      <Card className='rounded-2xl border-emerald-200/40 bg-emerald-500/5 py-3'>
        <CardContent className='flex items-center gap-3'>
          <Webhook className='size-4 text-emerald-500' />
          <div>
            <p className='text-2xl font-semibold tracking-tight'>{webhookCount}</p>
            <p className='text-xs text-muted-foreground'>
              <Trans>webhook endpoints</Trans>
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
