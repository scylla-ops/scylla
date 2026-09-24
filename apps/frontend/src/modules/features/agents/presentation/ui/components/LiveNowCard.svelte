<script lang="ts">
  import type { Snippet } from 'svelte';
  import ArrowUpRightIcon from '@lucide/svelte/icons/arrow-up-right';
  import { Card, CardContent } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AgentLiveJobs } from '../../utils/agent-mock-data.ts';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    jobs: AgentLiveJobs;
  }

  let { jobs }: Props = $props();
</script>

<!-- Not mounted yet: live jobs are mocked. See AGENTS.md. -->
{#snippet sectionHeader(tone: 'running' | 'pending', label: string, count: number)}
  {@const active = count > 0}
  <div class="flex items-center gap-2">
    <span class="relative flex h-2.5 w-2.5">
      {#if tone === 'running' && active}
        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70"
        ></span>
      {/if}
      <span
        class={cn(
          'relative inline-flex h-2.5 w-2.5 rounded-full',
          tone === 'running' ? 'bg-success' : 'bg-warning',
          !active && 'opacity-30',
        )}
      ></span>
    </span>
    <span class="font-mono text-xs uppercase tracking-wide text-muted-foreground">{label}</span>
    <span class="text-lg font-semibold leading-none">{count}</span>
  </div>
{/snippet}

{#snippet jobRow(
  tone: 'running' | 'pending',
  id: string,
  pipeline: string,
  line2: Snippet,
  accentValue: string,
)}
  <div
    class={cn(
      'group rounded border-l-[3px] p-2 pl-2.5 transition-colors',
      tone === 'running'
        ? 'border-l-success bg-success/10 hover:bg-success/5'
        : 'border-l-warning bg-warning/10 hover:bg-warning/5',
      'hover:border hover:border-l-[3px]',
      tone === 'running' ? 'hover:border-success' : 'hover:border-warning',
    )}
  >
    <div class="flex items-center gap-1.5 text-xs">
      <span class="font-mono font-semibold">{id.slice(0, 12)}…</span>
      <span class="text-muted-foreground">·</span>
      <span
        class={cn('truncate font-semibold', tone === 'running' ? 'text-success' : 'text-warning')}
      >
        {pipeline}
      </span>
      <ArrowUpRightIcon
        class="ml-auto h-3 w-3 shrink-0 opacity-40 transition-opacity group-hover:opacity-100"
      />
    </div>
    <div class="mt-0.5 font-mono text-[10px] text-muted-foreground">{@render line2()}</div>
    <span class="sr-only">{accentValue}</span>
  </div>
{/snippet}

<Card>
  <CardContent class="space-y-4 p-4">
    <div class="space-y-2">
      {@render sectionHeader('running', t(agentsMessages.running), jobs.running.length)}
      {#if jobs.running.length > 0}
        <div class="space-y-1.5">
          {#each jobs.running as job (job.id)}
            {#snippet runningLine()}
              {job.step} · <span class="text-success">{job.elapsed}</span>
            {/snippet}
            {@render jobRow('running', job.id, job.pipeline, runningLine, job.elapsed)}
          {/each}
        </div>
      {:else}
        <div
          class="rounded border border-dashed bg-muted/40 p-2 text-center text-xs text-muted-foreground"
        >
          {t(agentsMessages.idleNoJobs)}
        </div>
      {/if}
    </div>

    <div class="space-y-2">
      {@render sectionHeader('pending', t(agentsMessages.pending), jobs.pending.length)}
      {#if jobs.pending.length > 0}
        <div class="space-y-1.5">
          {#each jobs.pending as job (job.id)}
            {#snippet pendingLine()}
              {t(agentsMessages.waiting)}
              <span class="text-warning">{job.waiting}</span>
            {/snippet}
            {@render jobRow('pending', job.id, job.pipeline, pendingLine, job.waiting)}
          {/each}
        </div>
      {:else}
        <div
          class="rounded border border-dashed bg-muted/40 p-2 text-center text-xs text-muted-foreground"
        >
          {t(agentsMessages.queueEmpty)}
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
