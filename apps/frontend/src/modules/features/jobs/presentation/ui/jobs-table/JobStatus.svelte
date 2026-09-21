<script lang="ts">
  import { Badge } from '@shadcn-svelte';
  import { TruncatedText, getStatusIcon } from '@shared/presentation/ui-svelte';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import type { JobEntity } from '../../../domain/entities/job.entity.ts';
  import { jobsMessages } from '../jobs.messages.ts';

  interface Props {
    job: JobEntity;
  }

  let { job }: Props = $props();

  const config = $derived(getStatusConfig(job.status));
  const Icon = $derived(getStatusIcon(job.status));
</script>

<!-- A job's status badge: icon, label, and the one hint that explains a status
     the label alone leaves ambiguous. -->
<div class="w-full flex items-center gap-2">
  <Icon class={cn('w-5 h-5', config.iconClassName)} />
  <div class="flex min-w-0 flex-col">
    <Badge variant="outline" class={cn('w-fit', config.badgeClassName)}>
      {t(config.label)}
    </Badge>
    {#if job.status === 'pending'}
      <TruncatedText class="text-xs text-muted-foreground">
        {t(jobsMessages.queuedHint)}
      </TruncatedText>
    {:else if job.status === 'orphaned'}
      <TruncatedText class="text-xs text-muted-foreground">
        {t(jobsMessages.orphanedHint)}
      </TruncatedText>
    {/if}
  </div>
</div>
