<script lang="ts">
  import { i18n } from '@lingui/core';
  import CopyIcon from '@lucide/svelte/icons/copy';
  import CpuIcon from '@lucide/svelte/icons/cpu';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate } from '@platform/context';
  import {
    Badge,
    Button,
    buttonVariants,
    Card,
    CardContent,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { formatDate, getRelativeTime } from '@shared/utils/date-utils.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AgentEntity } from '../../../domain/entities/agent.entity.ts';
  import AgentIdLink from './AgentIdLink.svelte';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    agent: AgentEntity;
    onRequestDelete: (id: string) => void;
  }

  let { agent, onRequestDelete }: Props = $props();

  const canDelete = $derived(can(Permission.DELETE_APP));

  const copyId = async () => {
    await navigator.clipboard.writeText(agent.id);
    toast.success(i18n._(ToastMessages.AGENT_ID_COPIED));
  };
</script>

<Card
  class="cursor-pointer gap-0 py-0 transition-colors hover:bg-accent/50"
  onclick={() => scyllaNavigate.goToSubRoute(agent.id)}
>
  <CardContent class="p-4">
    <div class="flex items-start justify-between gap-2">
      <div class="flex min-w-0 items-center gap-3">
        <span
          class={cn(
            'relative flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-success/10',
            agent.connected ? 'border border-success' : 'border-2 border-destructive',
          )}
        >
          <CpuIcon class={cn('h-4 w-4', agent.connected ? 'text-success' : 'text-destructive')} />
          <!-- live status dot -->
          <span class="absolute -bottom-1 -right-1 flex h-3 w-3">
            {#if agent.connected}
              <span
                class="absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70"
              ></span>
            {/if}
            <span
              class={cn(
                'relative inline-flex h-3 w-3 rounded-full border-2 border-card',
                agent.connected ? 'bg-success' : 'bg-destructive',
              )}
            ></span>
          </span>
        </span>
        <div class="min-w-0">
          <p class="truncate font-semibold">{agent.name}</p>
          <AgentIdLink id={agent.id} truncate={18} />
        </div>
      </div>
      <DropdownMenu>
        <!-- The trigger *is* the button — see `AppCard` for why. -->
        <DropdownMenuTrigger
          class={cn(buttonVariants({ variant: 'ghost', size: 'icon' }), 'h-7 w-7 shrink-0')}
          onclick={(event: MouseEvent) => event.stopPropagation()}
        >
          <MoreHorizontalIcon class="h-4 w-4" />
          <span class="sr-only">{t(agentsMessages.agentActions)}</span>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" onclick={(event: MouseEvent) => event.stopPropagation()}>
          <DropdownMenuItem onSelect={() => void copyId()}>
            <CopyIcon class="mr-2 h-4 w-4" />
            {t(agentsMessages.copyId)}
          </DropdownMenuItem>
          {#if canDelete}
            <DropdownMenuItem variant="destructive" onSelect={() => onRequestDelete(agent.id)}>
              <TrashIcon class="mr-2 h-4 w-4" />
              {t(agentsMessages.delete)}
            </DropdownMenuItem>
          {/if}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <!-- status row -->
    <div class="mt-3 flex items-center justify-between gap-2">
      {#if agent.connected}
        <Badge class="gap-1 bg-success/15 text-success hover:bg-success/15">
          <span class="h-1.5 w-1.5 rounded-full bg-success"></span>
          {t(agentsMessages.online)}
        </Badge>
      {:else}
        <Badge variant="secondary" class="gap-1 text-destructive">
          <span class="h-1.5 w-1.5 rounded-full bg-destructive"></span>
          {t(agentsMessages.offline)}
        </Badge>
      {/if}
      <span class="font-mono text-xs text-muted-foreground">
        {#if !agent.lastSeen}
          {t(agentsMessages.neverConnected)}
        {:else if agent.connected}
          {t(agentsMessages.seen)}
          {getRelativeTime(agent.lastSeen)}
        {:else}
          {t(agentsMessages.down)}
          {getRelativeTime(agent.lastSeen)}
        {/if}
      </span>
    </div>
  </CardContent>

  <div class="flex items-center justify-between border-t border-dashed px-4 py-2.5">
    <span class="text-xs text-muted-foreground">
      {t(agentsMessages.created)}
      {formatDate(agent.createdAt)}
    </span>
    {#if canDelete}
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7 text-muted-foreground hover:text-destructive"
          onclick={event => {
            event.stopPropagation();
            onRequestDelete(agent.id);
          }}
        >
          <TrashIcon class="h-4 w-4" />
          <!-- Icon-only: the React original had no accessible name here. -->
          <span class="sr-only">{t(agentsMessages.deleteAgent)}</span>
        </Button>
      </div>
    {/if}
  </div>
</Card>
