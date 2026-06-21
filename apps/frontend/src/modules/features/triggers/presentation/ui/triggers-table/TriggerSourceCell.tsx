import { Clock, Webhook } from 'lucide-react';
import { Badge } from '@shadcn';
import { CodeSnippet } from '@shadcn/code-snippet.tsx';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { convertCronToLocal } from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';

/** Cron triggers show their expression; webhook triggers show their (copyable) URL. */
export const TriggerSourceCell = ({ trigger }: { trigger: TriggerEntity }) => {
  if (trigger.source.kind === TriggerKind.Cron) {
    return (
      <div className='flex items-center gap-2'>
        <Badge variant='secondary' className='gap-1'>
          <Clock className='size-3' /> Cron
        </Badge>
        <code className='font-mono text-xs text-muted-foreground'>
          {convertCronToLocal(trigger.source.expression) || '—'}
        </code>
      </div>
    );
  }

  return (
    <div className='flex flex-col items-start gap-1'>
      <Badge variant='secondary' className='gap-1'>
        <Webhook className='size-3' /> Webhook
      </Badge>
      {trigger.source.webhookUrl && (
        <CodeSnippet value={trigger.source.webhookUrl} className='w-full max-w-xs' />
      )}
    </div>
  );
};
