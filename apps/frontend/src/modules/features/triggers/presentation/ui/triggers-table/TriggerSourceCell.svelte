<script lang="ts">
  import { CopyableText } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
  import { TriggerKind } from '../../../domain/structs/trigger-source.struct.ts';
  import { convertCronToLocal } from '../../utils/trigger-form.utils.ts';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    trigger: TriggerEntity;
  }

  let { trigger }: Props = $props();
</script>

<!-- Cron triggers show their expression; webhook triggers show their (copyable) URL. -->
{#if trigger.source.kind === TriggerKind.Cron}
  <div class="flex items-center justify-center gap-2">
    <code class="font-mono text-xs text-muted-foreground">
      {convertCronToLocal(trigger.source.expression) || '—'}
    </code>
  </div>
{:else if trigger.source.kind === TriggerKind.Webhook}
  <CopyableText
    value={trigger.source.webhookUrl}
    showFullOnHover
    copyLabel={t(triggersMessages.copyUrl)}
    class="w-full justify-center"
  />
{:else}
  <!-- A source arm this build doesn't know: nothing meaningful to render. -->
  <div class="flex items-center justify-center">
    <code class="font-mono text-xs text-muted-foreground">—</code>
  </div>
{/if}
