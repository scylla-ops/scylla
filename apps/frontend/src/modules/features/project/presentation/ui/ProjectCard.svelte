<script lang="ts">
  import FolderIcon from '@lucide/svelte/icons/folder';
  import PencilIcon from '@lucide/svelte/icons/pencil';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate } from '@platform/context';
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    Checkbox,
    Tooltip,
    TooltipContent,
    TooltipTrigger,
  } from '@shadcn-svelte';
  import { IconButton } from '@shared/presentation/ui-svelte';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { ProjectEntity } from '../../domain/entities/project.entity.ts';
  import EditProjectDialog from './EditProjectDialog.svelte';
  import { projectMessages } from './project.messages.ts';

  interface Props {
    project: ProjectEntity;
  }

  let { project }: Props = $props();

  const selection = createSelection('projects');
  const isSelected = $derived(selection.selectedIds.includes(project.id));

  let editOpen = $state(false);
</script>

<Card
  onclick={() => scyllaNavigate.goToProject(project.id, project.name)}
  class={cn(
    'group flex h-full cursor-pointer flex-col transition-all duration-200 hover:border-primary/50 hover:shadow-lg active:scale-[0.98]',
    isSelected && 'border-primary ring-2 ring-primary',
  )}
>
  <CardHeader class="pb-2">
    <div class="flex items-start justify-between gap-2">
      <div class="flex min-w-0 flex-1 items-center gap-3">
        <div class="shrink-0 rounded-lg bg-primary/10 p-2 transition-colors group-hover:bg-primary/20">
          <FolderIcon class="h-5 w-5 text-primary" />
        </div>
        <CardTitle class="min-w-0 truncate text-base font-semibold" title={project.name}>
          {project.name}
        </CardTitle>
      </div>
      <div class="flex shrink-0 flex-row items-center gap-1">
        <!-- Checked against this card's project, not the active one. -->
        {#if can(Permission.UPDATE_PROJECT, { projectId: project.id })}
          <IconButton
            icon={PencilIcon}
            tooltip={t(projectMessages.edit)}
            onclick={event => {
              event.stopPropagation();
              editOpen = true;
            }}
            class="h-7 w-7 sm:opacity-0 sm:group-hover:opacity-100 sm:focus-visible:opacity-100"
            iconClass="h-3.5 w-3.5"
          />
        {/if}
        <Tooltip>
          <TooltipTrigger>
            {#snippet child({ props })}
              <span {...props} class="mr-2">
                <Checkbox
                  checked={isSelected}
                  class="cursor-pointer transition-all hover:scale-125 hover:border-primary"
                  onclick={event => event.stopPropagation()}
                  onCheckedChange={() => selection.select(project.id)}
                />
              </span>
            {/snippet}
          </TooltipTrigger>
          <TooltipContent>
            <p>{t(projectMessages.select)}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  </CardHeader>

  <CardContent class="flex-1 pt-0">
    <p class="line-clamp-2 text-sm text-muted-foreground">
      {#if project.description}
        {project.description}
      {:else}
        <span class="italic">{t(projectMessages.noDescription)}</span>
      {/if}
    </p>
  </CardContent>
</Card>

<EditProjectDialog open={editOpen} setOpen={open => (editOpen = open)} {project} />
