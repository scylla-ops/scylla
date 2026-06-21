import { Check, Copy } from 'lucide-react';
import { Button } from '@shadcn';
import { TriggerKind } from '@/modules/features/triggers/domain/models/trigger-source.model.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { convertCronToLocal } from '@/modules/features/triggers/presentation/utils/trigger-form.utils.ts';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';
import { useState } from 'react';

/** Cron triggers show their expression; webhook triggers show their (copyable) URL. */
export const TriggerSourceCell = ({ trigger }: { trigger: TriggerEntity }) => {
  const [copied, setCopied] = useState(false);

  if (trigger.source.kind === TriggerKind.Cron) {
    return (
      <div className='flex items-center justify-center gap-2'>
        <code className='font-mono text-xs text-muted-foreground'>
          {convertCronToLocal(trigger.source.expression) || '—'}
        </code>
      </div>
    );
  }

  const handleCopyUrl = (e: React.MouseEvent) => {
    e.stopPropagation();

    if (trigger.source.kind === TriggerKind.Webhook)
      void navigator.clipboard.writeText(trigger.source.webhookUrl);

    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className='w-full flex items-center justify-center gap-2'>
      {trigger.source.webhookUrl && (
        <div className='flex items-center gap-2'>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className='font-mono text-sm truncate'>
                {trigger.source.webhookUrl.slice(0, 12)}...
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p>{trigger.source.webhookUrl}</p>
            </TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size='icon'
                variant='ghost'
                className='h-6 w-6 shrink-0 cursor-pointer transition-all hover:scale-125 hover:text-primary hover:bg-primary-hover rounded-full'
                onClick={handleCopyUrl}
              >
                {copied ? (
                  <Check className='w-3 h-3 text-green-500' />
                ) : (
                  <Copy className='w-3 h-3' />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{copied ? <Trans>Copied!</Trans> : <Trans>Copy url</Trans>}</p>
            </TooltipContent>
          </Tooltip>
        </div>
      )}
    </div>
  );
};
