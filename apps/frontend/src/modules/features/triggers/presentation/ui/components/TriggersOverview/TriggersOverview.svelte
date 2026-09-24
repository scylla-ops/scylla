<script lang="ts">
  import ClockIcon from '@lucide/svelte/icons/clock';
  import PowerIcon from '@lucide/svelte/icons/power';
  import WebhookIcon from '@lucide/svelte/icons/webhook';
  import { Card, CardContent } from '@shadcn';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { TriggerEntity } from '../../../../domain/entities/trigger.entity.ts';
  import { TriggerKind } from '../../../../domain/structs/trigger-source.struct.ts';
  import { triggersMessages } from '../../triggers.messages.ts';

  interface Props {
    triggers: TriggerEntity[];
  }

  let { triggers }: Props = $props();

  const enabledCount = $derived(triggers.filter(trigger => trigger.enabled).length);
  const webhookCount = $derived(
    triggers.filter(trigger => trigger.source.kind === TriggerKind.Webhook).length,
  );
  const nextFire = $derived(
    triggers
      .filter(
        trigger =>
          trigger.enabled && trigger.source.kind === TriggerKind.Cron && !!trigger.nextFireAt,
      )
      .map(trigger => trigger.nextFireAt as string)
      .sort((left, right) => left.localeCompare(right))[0],
  );
</script>

<div class="grid gap-4 md:grid-cols-3">
  <Card class="rounded-2xl border-primary/20 bg-primary/5 py-3">
    <CardContent class="flex items-center gap-3">
      <PowerIcon class="size-4 text-primary" />
      <div>
        <p class="text-2xl font-semibold tracking-tight">{enabledCount}/{triggers.length}</p>
        <p class="text-xs text-muted-foreground">{t(triggersMessages.enabled)}</p>
      </div>
    </CardContent>
  </Card>

  <Card class="rounded-2xl border-blue-200/40 bg-blue-500/5 py-3">
    <CardContent class="h-full w-full flex items-center gap-3">
      <ClockIcon class="size-4 text-blue-500" />
      <div>
        <p class="text-sm font-semibold tracking-tight">
          {nextFire ? formatDate(nextFire) : '—'}
        </p>
        <p class="text-xs text-muted-foreground">{t(triggersMessages.nextScheduledRun)}</p>
      </div>
    </CardContent>
  </Card>

  <Card class="rounded-2xl border-emerald-200/40 bg-emerald-500/5 py-3">
    <CardContent class="flex items-center gap-3">
      <WebhookIcon class="size-4 text-emerald-500" />
      <div>
        <p class="text-2xl font-semibold tracking-tight">{webhookCount}</p>
        <p class="text-xs text-muted-foreground">{t(triggersMessages.webhookEndpoints)}</p>
      </div>
    </CardContent>
  </Card>
</div>
