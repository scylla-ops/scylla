import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { convertCronToLocal } from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';
import { CopyableText } from '@shared/presentation/ui/data-display/CopyableText.tsx';
import { Trans } from '@lingui/react/macro';

/** Cron triggers show their expression; webhook triggers show their (copyable) URL. */
export const TriggerSourceCell = ({ trigger }: { trigger: TriggerEntity }) => {
  if (trigger.source.kind === TriggerKind.Cron) {
    return (
      <div className='flex items-center justify-center gap-2'>
        <code className='font-mono text-xs text-muted-foreground'>
          {convertCronToLocal(trigger.source.expression) || '—'}
        </code>
      </div>
    );
  }

  if (!trigger.source.webhookUrl) return <div className='w-full' />;

  return (
    <CopyableText
      value={trigger.source.webhookUrl}
      truncate={12}
      showFullOnHover
      copyLabel={<Trans>Copy url</Trans>}
      className='w-full justify-center'
    />
  );
};
