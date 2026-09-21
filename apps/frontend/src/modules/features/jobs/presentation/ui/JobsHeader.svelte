<script lang="ts">
  import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
  import { Permission, can } from '@platform/authz';
  import { createMutation } from '@platform/query';
  import { Button, Tooltip, TooltipContent, TooltipTrigger } from '@shadcn-svelte';
  import { CopyableText, FeatureHeader } from '@shared/presentation/ui-svelte';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { jobMutations } from '../jobs.queries.ts';
  import { jobsMessages } from './jobs.messages.ts';

  interface Props {
    numberOfJobs: number;
    jobIds: string[];
    pipelineId: string;
    onRefresh: () => void;
    /**
     * Runs the pipeline these jobs belong to. Injected by whoever owns the
     * route, because running is a pipeline operation — jobs would otherwise
     * have to import the pipeline module that already reads jobs.
     */
    onRun?: () => Promise<void>;
  }

  let { numberOfJobs, jobIds, pipelineId, onRefresh, onRun }: Props = $props();

  const deleteJob = createMutation(() => jobMutations.remove(pipelineId));

  // A getter, not the array: the ids are server data that arrives late, and
  // capturing them once would freeze "select all" on the first, empty render.
  const selection = createFeatureSelection('jobs', () => jobIds, {
    deleteItem: id => deleteJob.mutateAsync(id),
  });

  const canRun = $derived(can(Permission.RUN_PIPELINE));
  const canDelete = $derived(can(Permission.DELETE_JOB));
</script>

{#snippet pipelineIdLine()}
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="text-sm text-muted-foreground font-medium">
        {t(jobsMessages.pipelineIdLabel)}
      </span>
      <CopyableText value={pipelineId} class="text-sm text-muted-foreground font-medium" />
    </div>
  </div>
{/snippet}

{#snippet refreshAction()}
  <Tooltip>
    <TooltipTrigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="outline"
          size="icon"
          onclick={onRefresh}
          class="h-9 w-9 cursor-pointer transition-all hover:scale-110"
        >
          <RefreshCwIcon class="size-4" />
          <!-- The tooltip is only `aria-describedby`: without this the button
               has no accessible name while it is closed. -->
          <span class="sr-only">{t(jobsMessages.refresh)}</span>
        </Button>
      {/snippet}
    </TooltipTrigger>
    <TooltipContent>
      <p>{t(jobsMessages.refresh)}</p>
    </TooltipContent>
  </Tooltip>
{/snippet}

<div class="flex flex-col gap-3">
  <FeatureHeader
    count={numberOfJobs}
    label={t(jobsMessages.job)}
    pluralLabel={t(jobsMessages.jobs)}
    newLabel={t(jobsMessages.run)}
    onNew={onRun ? () => void onRun() : undefined}
    canNew={canRun}
    newDeniedReason={t(jobsMessages.runDenied)}
    {canDelete}
    deleteDeniedReason={t(jobsMessages.deleteDenied)}
    underLabel={pipelineIdLine}
    extraActions={refreshAction}
    {...selection.headerProps}
  />
</div>
