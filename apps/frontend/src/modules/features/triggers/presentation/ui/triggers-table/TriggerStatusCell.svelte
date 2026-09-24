<script lang="ts">
  import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
  import HelpCircleIcon from '@lucide/svelte/icons/help-circle';
  import MinusCircleIcon from '@lucide/svelte/icons/minus-circle';
  import XCircleIcon from '@lucide/svelte/icons/x-circle';
  import { Badge } from '@shadcn';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
  import { TriggerKind } from '../../../domain/structs/trigger-source.struct.ts';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    trigger: TriggerEntity;
  }

  let { trigger }: Props = $props();

  const result = $derived(trigger.lastResult);
  const hasFired = $derived(!!result || !!trigger.lastFiredAt);
</script>

<!-- Last-run badge (shared status language) plus a context line: next fire or
     the reason it is off. -->
<div class="flex flex-col items-center justify-center gap-1">
  {#if !hasFired}
    <Badge variant="outline" class="gap-1">
      <MinusCircleIcon class="size-3" />
      {t(triggersMessages.neverFired)}
    </Badge>
  {:else if result?.kind === 'failed'}
    <Badge variant="destructive" class="gap-1" title={result.error}>
      <XCircleIcon class="size-3" />
      {t(triggersMessages.error)}
    </Badge>
  {:else if result?.kind === 'succeeded'}
    <Badge variant="default" class="gap-1">
      <CheckCircle2Icon class="size-3" />
      {t(triggersMessages.ok)}
    </Badge>
  {:else}
    <!-- Fired, but the outcome arm is newer than this build — don't claim success. -->
    <Badge variant="outline" class="gap-1">
      <HelpCircleIcon class="size-3" />
      {t(triggersMessages.unknown)}
    </Badge>
  {/if}

  {#if !trigger.enabled}
    <span class="text-xs text-muted-foreground">
      {trigger.source.kind === TriggerKind.Webhook
        ? t(triggersMessages.disabledWebhook)
        : t(triggersMessages.disabled)}
    </span>
  {:else if trigger.source.kind === TriggerKind.Cron}
    {#if trigger.nextFireAt}
      <span class="text-xs text-muted-foreground">
        {t(triggersMessages.next)}
        {formatDate(trigger.nextFireAt)}
      </span>
    {/if}
  {:else if trigger.lastFiredAt}
    <span class="text-xs text-muted-foreground">
      {t(triggersMessages.last)}
      {formatDate(trigger.lastFiredAt)}
    </span>
  {/if}
</div>
