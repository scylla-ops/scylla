<script lang="ts">
  import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
  import { Button } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { mockInitialLogs, type LogLevel } from '../../utils/agent-mock-data.ts';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    agentId: string;
    agentName: string;
  }

  let { agentId, agentName }: Props = $props();

  const LEVEL_COLOR: Record<LogLevel, string> = {
    info: '#9aa0a6',
    ok: '#7adfa1',
    warn: '#f0c460',
    error: '#f08581',
  };

  const lines = $derived(mockInitialLogs(agentId));
</script>

<!--
  Static preview only, and nothing mounts it yet: the agent log stream does not
  exist. Ported rather than deleted — see this module's AGENTS.md.
-->
<div class="space-y-2">
  <div class="flex items-center justify-between gap-2">
    <div class="flex items-baseline gap-2">
      <h2 class="text-lg font-semibold">{t(agentsMessages.logs)}</h2>
      <span class="font-mono text-xs text-muted-foreground">
        {t(agentsMessages.logsNotWired)}
      </span>
    </div>
    <div class="flex items-center gap-2">
      <span
        class="rounded-full border bg-muted px-2 py-0.5 text-[11px] font-mono text-muted-foreground"
      >
        {t(agentsMessages.sample)}
      </span>
      <Button variant="outline" size="sm" disabled title={t(agentsMessages.comingSoon)}>
        {t(agentsMessages.filter)} ▾
      </Button>
      <Button variant="outline" size="sm" disabled title={t(agentsMessages.comingSoon)}>
        {t(agentsMessages.openFullLogs)}
        <ExternalLinkIcon class="ml-1 h-3 w-3" />
      </Button>
    </div>
  </div>

  <div class="overflow-hidden rounded-md" style="background: #1a1816">
    <div
      class="flex items-center justify-between px-3 py-1.5 font-mono text-[11px]"
      style="border-bottom: 1px solid rgba(255,255,255,0.08); color: #9aa0a6"
    >
      <span>~/agent/{agentName}.log</span>
      <span>{lines.length} {t(agentsMessages.lines)}</span>
    </div>
    <div class="max-h-[240px] overflow-y-auto px-3 py-2">
      {#each lines as line, index (index)}
        <div
          class="grid items-baseline gap-2.5 py-px font-mono text-[11px]"
          style="grid-template-columns: 76px 50px 1fr"
        >
          <span style="color: #666">{line.t}</span>
          <span style={`color: ${LEVEL_COLOR[line.level]}`}>{line.level.toUpperCase()}</span>
          <span class="truncate" style="color: #e8e6e2; white-space: nowrap">{line.msg}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
